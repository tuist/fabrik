// Fabrik Website - Interactive Elements

(function() {
  'use strict';

  // Navbar scroll effect
  const navbar = document.querySelector('.navbar');
  let lastScroll = 0;

  function handleScroll() {
    const currentScroll = window.pageYOffset;

    if (currentScroll > 100) {
      navbar.style.backgroundColor = 'rgba(13, 14, 18, 0.95)';
      navbar.style.boxShadow = '0 2px 16px rgba(0, 0, 0, 0.3)';
    } else {
      navbar.style.backgroundColor = 'rgba(13, 14, 18, 0.8)';
      navbar.style.boxShadow = 'none';
    }

    lastScroll = currentScroll;
  }

  // Smooth scroll for anchor links
  document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function(e) {
      const href = this.getAttribute('href');
      if (href === '#') return;

      e.preventDefault();
      const target = document.querySelector(href);
      if (target) {
        const offsetTop = target.offsetTop - 80;
        window.scrollTo({
          top: offsetTop,
          behavior: 'smooth'
        });
      }
    });
  });

  // Intersection Observer for fade-in animations
  const observerOptions = {
    threshold: 0.1,
    rootMargin: '0px 0px -50px 0px'
  };

  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        entry.target.style.opacity = '1';
        entry.target.style.transform = 'translateY(0)';
      }
    });
  }, observerOptions);

  // Animate feature cards on scroll
  document.querySelectorAll('.feature-card, .layer').forEach(card => {
    card.style.opacity = '0';
    card.style.transform = 'translateY(20px)';
    card.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
    observer.observe(card);
  });

  // Copy code to clipboard
  document.querySelectorAll('.code-block').forEach(block => {
    block.style.position = 'relative';

    const button = document.createElement('button');
    button.className = 'copy-button';
    button.innerHTML = `
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
      </svg>
    `;
    button.style.cssText = `
      position: absolute;
      top: 12px;
      right: 12px;
      background: var(--bg-secondary);
      border: 1px solid var(--border-color);
      border-radius: 6px;
      padding: 6px 10px;
      color: var(--text-secondary);
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 0.875rem;
      transition: all 0.15s ease;
      z-index: 10;
    `;

    button.addEventListener('mouseenter', () => {
      button.style.background = 'var(--bg-tertiary)';
      button.style.borderColor = 'var(--border-hover)';
    });

    button.addEventListener('mouseleave', () => {
      button.style.background = 'var(--bg-secondary)';
      button.style.borderColor = 'var(--border-color)';
    });

    button.addEventListener('click', async () => {
      const code = block.querySelector('pre').textContent;
      try {
        await navigator.clipboard.writeText(code);
        button.innerHTML = `
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
          Copied!
        `;
        button.style.color = 'var(--accent-primary)';

        setTimeout(() => {
          button.innerHTML = `
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
          `;
          button.style.color = 'var(--text-secondary)';
        }, 2000);
      } catch (err) {
        console.error('Failed to copy:', err);
      }
    });

    block.appendChild(button);
  });

  // Initialize scroll handler
  window.addEventListener('scroll', handleScroll, { passive: true });
  handleScroll();

  // Stagger animation for feature cards
  document.querySelectorAll('.feature-card').forEach((card, index) => {
    card.style.transitionDelay = `${index * 0.05}s`;
  });
})();
