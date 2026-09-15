(() => {
  'use strict';

  /* ---------- mobile nav toggle ---------- */
  const navToggle = document.getElementById('navToggle');
  const navLinks = document.getElementById('navLinks');

  if (navToggle && navLinks) {
    navToggle.addEventListener('click', () => {
      const isOpen = navLinks.classList.toggle('is-open');
      navToggle.setAttribute('aria-expanded', String(isOpen));
    });

    navLinks.querySelectorAll('a').forEach((link) => {
      link.addEventListener('click', () => {
        navLinks.classList.remove('is-open');
        navToggle.setAttribute('aria-expanded', 'false');
      });
    });
  }

  /* ---------- copy-to-clipboard buttons ---------- */
  document.querySelectorAll('.copy-btn').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const text = btn.getAttribute('data-copy') || '';
      const original = btn.textContent;

      try {
        await navigator.clipboard.writeText(text);
      } catch (err) {
        // fallback for older browsers / non-secure contexts
        const helper = document.createElement('textarea');
        helper.value = text;
        helper.style.position = 'fixed';
        helper.style.opacity = '0';
        document.body.appendChild(helper);
        helper.select();
        document.execCommand('copy');
        document.body.removeChild(helper);
      }

      btn.textContent = 'Copied!';
      btn.classList.add('is-copied');
      setTimeout(() => {
        btn.textContent = original;
        btn.classList.remove('is-copied');
      }, 1800);
    });
  });

  /* ---------- generic tab switcher (demo + install) ---------- */
  function initTabs(tabSelector, panelSelector) {
    const tabs = document.querySelectorAll(tabSelector);
    tabs.forEach((tab) => {
      tab.addEventListener('click', () => {
        const targetId = tab.getAttribute('data-target');
        const group = tab.parentElement;

        group.querySelectorAll(tabSelector).forEach((t) => t.classList.remove('is-active'));
        tab.classList.add('is-active');

        document.querySelectorAll(panelSelector).forEach((panel) => {
          const isTarget = panel.id === targetId;
          panel.classList.toggle('is-active', isTarget);

          const video = panel.querySelector('video');
          if (video) {
            if (isTarget) {
              video.setAttribute('preload', 'auto');
              video.play().catch(() => {});
            } else {
              video.pause();
            }
          }
        });
      });
    });
  }

  initTabs('.demo-tab', '.demo-panel');
  initTabs('.install-tab', '.install-panel');

  /* ---------- reveal-on-scroll ---------- */
  const revealEls = document.querySelectorAll('.reveal');
  if ('IntersectionObserver' in window && revealEls.length) {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add('is-visible');
            observer.unobserve(entry.target);
          }
        });
      },
      { threshold: 0.12, rootMargin: '0px 0px -40px 0px' }
    );
    revealEls.forEach((el) => observer.observe(el));
  } else {
    revealEls.forEach((el) => el.classList.add('is-visible'));
  }

  /* ---------- hero terminal typing effect ---------- */
  const typedLineEl = document.getElementById('typedLine');
  const typedOutputEl = document.getElementById('typedOutput');

  const heroQuery = 'sage "compress all png files in this directory"';
  const heroOutput = [
    { text: '$ find . -name "*.png" -exec pngquant --ext .png {} \\;', cls: 'cmd-line' },
    { text: '# find: searches the filesystem for files', cls: 'hint-line' },
    { text: '# -name "*.png": matches files ending in .png', cls: 'hint-line' },
    { text: '# -exec ...: runs pngquant on each match', cls: 'hint-line' },
    { text: "✓ Copied to clipboard. Press '⌘ + V', then Enter to run.", cls: 'ok-line' },
  ];

  function typeText(el, text, speed) {
    return new Promise((resolve) => {
      let i = 0;
      const tick = () => {
        if (i <= text.length) {
          el.textContent = text.slice(0, i);
          i += 1;
          setTimeout(tick, speed);
        } else {
          resolve();
        }
      };
      tick();
    });
  }

  async function playHeroAnimation() {
    if (!typedLineEl || !typedOutputEl) return;

    typedLineEl.textContent = '';
    typedOutputEl.innerHTML = '';

    await typeText(typedLineEl, heroQuery, 35);
    await new Promise((r) => setTimeout(r, 350));

    for (const line of heroOutput) {
      const p = document.createElement('p');
      p.className = line.cls;
      typedOutputEl.appendChild(p);
      await typeText(p, line.text, 12);
      await new Promise((r) => setTimeout(r, 120));
    }

    await new Promise((r) => setTimeout(r, 2600));
    playHeroAnimation();
  }

  playHeroAnimation();

  /* ---------- active nav link highlighting ---------- */
  const sections = document.querySelectorAll('main section[id]');
  const navAnchors = document.querySelectorAll('.nav-link[href^="#"]');

  if ('IntersectionObserver' in window && sections.length) {
    const navObserver = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            navAnchors.forEach((a) => {
              a.style.color = a.getAttribute('href') === `#${entry.target.id}` ? 'var(--neon-cyan)' : '';
            });
          }
        });
      },
      { rootMargin: '-45% 0px -50% 0px' }
    );
    sections.forEach((s) => navObserver.observe(s));
  }
})();
