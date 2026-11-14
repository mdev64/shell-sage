use anyhow::Result;
use llama_cpp_2::LogOptions;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use std::num::NonZeroU32;
use std::path::Path;

/// Context window (prompt + generated tokens).
pub const CONTEXT_SIZE: u32 = 1024;

// Repetition-penalty settings that keep the small models out of loops.
const PENALTY_LAST_N: i32 = 64;
const PENALTY_REPEAT: f32 = 1.1;

pub struct InferenceEngine {
    // Field order is drop order: the model must be freed before the backend.
    model: LlamaModel,
    backend: LlamaBackend,
}

impl InferenceEngine {
    pub fn new(model_path: &Path) -> Result<Self> {
        llama_cpp_2::send_logs_to_tracing(LogOptions::default().with_logs_enabled(false));
        let backend = LlamaBackend::init()?;
        let model = LlamaModel::load_from_file(&backend, model_path, &LlamaModelParams::default())?;
        Ok(Self { model, backend })
    }

    /// Greedily generates a completion for `prompt`, stopping at an
    /// end-of-generation token or once `max_tokens` total positions
    /// (prompt included) have been used.
    pub fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        let tokens = self.model.str_to_token(prompt, AddBos::Always)?;
        // The whole prompt must fit in the context with room left to answer.
        let limit = max_tokens.min(CONTEXT_SIZE as usize);
        if tokens.len() >= limit {
            anyhow::bail!(
                "Input is too long ({} tokens; the limit is {} including instructions). Please shorten it.",
                tokens.len(),
                limit - 1
            );
        }

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(CONTEXT_SIZE))
            .with_n_batch(CONTEXT_SIZE);
        let mut ctx = self.model.new_context(&self.backend, ctx_params)?;

        let mut batch = LlamaBatch::new(CONTEXT_SIZE as usize, 1);
        let last = tokens.len().saturating_sub(1);
        for (i, &token) in tokens.iter().enumerate() {
            batch.add(token, i as i32, &[0], i == last)?;
        }
        ctx.decode(&mut batch)?;

        let mut penalty = LlamaSampler::penalties(PENALTY_LAST_N, PENALTY_REPEAT, 0.0, 0.0);
        penalty.accept_many(tokens.iter().copied());

        // One decoder for the whole stream so multi-byte UTF-8 characters
        // split across tokens are reassembled correctly.
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut output = String::new();
        let mut n_cur = batch.n_tokens();

        while (n_cur as usize) < limit {
            let mut candidates =
                LlamaTokenDataArray::from_iter(ctx.candidates_ith(batch.n_tokens() - 1), false);
            candidates.apply_sampler(&penalty);
            let token = candidates.sample_token_greedy();

            if self.model.is_eog_token(token) {
                break;
            }
            penalty.accept(token);
            output.push_str(
                &self
                    .model
                    .token_to_piece(token, &mut decoder, false, None)?,
            );

            batch.clear();
            batch.add(token, n_cur, &[0], true)?;
            ctx.decode(&mut batch)?;
            n_cur += 1;
        }

        Ok(output)
    }
}
