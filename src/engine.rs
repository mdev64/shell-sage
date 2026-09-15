use anyhow::Result;
use llama_cpp_2::LogOptions;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use std::path::Path;

pub struct InferenceEngine {
    backend: LlamaBackend,
    model: LlamaModel,
}

impl InferenceEngine {
    pub fn new(model_path: &Path) -> Result<Self> {
        llama_cpp_2::send_logs_to_tracing(LogOptions::default().with_logs_enabled(false));
        let backend = LlamaBackend::init()?;
        let model = LlamaModel::load_from_file(&backend, model_path, &LlamaModelParams::default())?;
        Ok(Self { backend, model })
    }

    pub fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(std::num::NonZeroU32::new(1024).unwrap()));
        let mut ctx = self.model.new_context(&self.backend, ctx_params)?;

        let tokens = self.model.str_to_token(prompt, AddBos::Always)?;
        let mut batch = LlamaBatch::new(1024, 1);

        for (i, &token) in tokens.iter().enumerate() {
            batch.add(token, i as i32, &[0], i == tokens.len() - 1)?;
        }
        ctx.decode(&mut batch)?;

        let mut penalty = LlamaSampler::penalties(64, 1.1, 0.0, 0.0);
        penalty.accept_many(tokens.iter().copied());

        let mut output = String::new();
        let mut n_cur = batch.n_tokens();

        while (n_cur as usize) < max_tokens {
            let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
            let mut candidates_p = LlamaTokenDataArray::from_iter(candidates, false);
            candidates_p.apply_sampler(&penalty);
            let token = candidates_p.sample_token_greedy();

            if self.model.is_eog_token(token) {
                break;
            }

            penalty.accept(token);

            let mut decoder = encoding_rs::UTF_8.new_decoder();
            let piece = self
                .model
                .token_to_piece(token, &mut decoder, false, None)?;
            output.push_str(&piece);

            batch.clear();
            batch.add(token, n_cur, &[0], true)?;
            ctx.decode(&mut batch)?;
            n_cur += 1;
        }

        Ok(output)
    }
}
