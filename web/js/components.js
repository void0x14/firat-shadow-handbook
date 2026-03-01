/**
 * Fırat Shadow Handbook - UI Components
 * Professional, minimal UI - Zero dependency
 *
 * Security: XSS Prevention - All dynamic content must be escaped
 */

// Security: HTML escaping utility to prevent XSS
function escapeHtml(text) {
    if (text === null || text === undefined) return '';
    const div = document.createElement('div');
    div.textContent = String(text);
    return div.innerHTML;
}

// Component base class
class Component {
    constructor(container) {
        this.container = typeof container === 'string'
            ? document.querySelector(container)
            : container;
        this.state = {};
        this.unsubscribers = [];
    }

    setState(newState) {
        const oldState = { ...this.state };
        this.state = { ...this.state, ...newState };
        if (this.shouldUpdate(oldState, this.state)) {
            this.render();
        }
    }

    shouldUpdate(oldState, newState) {
        return JSON.stringify(oldState) !== JSON.stringify(newState);
    }

    render() {
        if (this.container) {
            // Security: Template output is trusted (written by developer)
            // But any user-generated data in template() must use escapeHtml()
            this.container.innerHTML = this.template();
            this.bindEvents();
        }
    }

    template() {
        return '';
    }

    bindEvents() {}

    subscribe(store, key, fn) {
        const unsub = store.subscribe(key, fn);
        this.unsubscribers.push(unsub);
        return unsub;
    }

    destroy() {
        this.unsubscribers.forEach(unsub => unsub());
    }
}

// Dashboard Page Component
class DashboardPage extends Component {
    template() {
        const user = store.get('user');
        const role = store.get('role');

        // Security: Escape all dynamic content that could contain user input
        const userName = escapeHtml(user?.name);
        const userRole = escapeHtml(role);
        
        return `
            <div class="dashboard">
                <!-- Page Header -->
                <div class="page-header">
                    <div class="page-header__content">
                        <h1 class="page-header__title">${t('dashboard.welcome')}, ${userName || t('user.guest')}</h1>
                        <p class="page-header__subtitle">${t('dashboard.subtitle')}</p>
                    </div>
                </div>

                <!-- Empty State -->
                <div class="empty-state">
                    <div class="empty-state__icon">📚</div>
                    <h2 class="empty-state__title">${t('dashboard.welcome')}</h2>
                    <p class="empty-state__text">Fırat Shadow Handbook'a hoş geldiniz. Dersleriniz ve kayıtlarınız burada görünecek.</p>
                </div>

                <!-- Two Column Layout -->
                <div class="dashboard-columns">
                    <!-- Courses Section -->
                    <section class="section section--courses">
                        <div class="section__header">
                            <h2 class="section__title">${t('dashboard.myCourses')}</h2>
                            <a href="#/courses" class="link-arrow">${t('common.viewAll')}</a>
                        </div>
                        <div class="courses-grid">
                            <div class="empty-state">
                                <p class="empty-state__text">Henüz ders bulunmuyor.</p>
                            </div>
                        </div>
                    </section>

                    <!-- Sidebar Widgets -->
                    <aside class="sidebar-widgets">
                        ${this.renderQuickActions(userRole)}
                    </aside>
                </div>
            </div>
        `;
    }

    renderQuickActions(role) {
        return `
            <div class="widget">
                <h3 class="widget__title">${t('dashboard.quickActions')}</h3>
                <div class="widget__actions">
                    ${role === 'teacher' ? `
                        <button class="action-btn action-btn--primary" onclick="app.startClass()">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <polygon points="5 3 19 12 5 21 5 3"></polygon>
                            </svg>
                            <span>${t('teacher.startClass')}</span>
                        </button>
                    ` : `
                        <a href="#/recordings" class="action-btn">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <polygon points="23 7 16 12 23 17 23 7"></polygon>
                                <rect x="1" y="5" width="15" height="14" rx="2" ry="2"></rect>
                            </svg>
                            <span>${t('nav.recordings')}</span>
                        </a>
                    `}
                </div>
            </div>
        `;
    }
}

// Login Page Component
class LoginPage extends Component {
    template() {
        return `
            <div class="login-page">
                <div class="login-card">
                    <div class="login-card__header">
                        <div class="login-logo">
                            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                <path d="M12 2L2 7l10 5 10-5-10-5z"></path>
                                <path d="M2 17l10 5 10-5"></path>
                                <path d="M2 12l10 5 10-5"></path>
                            </svg>
                        </div>
                        <h1 class="login-title">Fırat Shadow Handbook</h1>
                        <p class="login-subtitle">Fırat Üniversitesi Öğrenci Portalı</p>
                    </div>
                    
                    <div class="login-card__body">
                        <div class="login-info">
                            <p>Bu sistem, Fırat Üniversitesi öğrencileri için ders kayıtlarını ve canlı dersleri takip etmeyi sağlar.</p>
                        </div>
                        
                        <button id="casLoginBtn" class="login-btn login-btn--primary">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"></path>
                                <polyline points="10 17 15 12 10 7"></polyline>
                                <line x1="15" y1="12" x2="3" y2="12"></line>
                            </svg>
                            <span>Fırat Üniversitesi ile Giriş Yap</span>
                        </button>
                        
                        <p class="login-note">
                            Giriş yapmak için Fırat Üniversitesi OBS hesabınızı kullanın.
                        </p>
                    </div>
                </div>
            </div>
        `;
    }
    
    bindEvents() {
        const btn = document.getElementById('casLoginBtn');
        if (btn) {
            btn.addEventListener('click', () => {
                // Redirect to Fırat University CAS login
                const casUrl = 'https://jasig.firat.edu.tr/cas/login';
                const serviceUrl = encodeURIComponent('https://debsis.firat.edu.tr/login/index.php?authCAS=CAS');
                window.location.href = `${casUrl}?service=${serviceUrl}`;
            });
        }
    }
}

// Sazan.avi Modal
function showSazanModal() {
    const currentMode = store.get('sazanMode');
    const modes = [
        { id: 0, name: 'Kapalı', desc: 'Sazan.avi devre dışı' },
        { id: 1, name: 'Manuel', desc: 'Sadece siz tetiklediğinde çalışır' },
        { id: 2, name: 'Yarı Otomatik', desc: 'Soruları algılar, onayınızı bekler' },
        { id: 3, name: 'Tam Otomatik', desc: 'Soruları algılar ve yanıtlar' },
        { id: 4, name: 'AI Modu', desc: 'LLM ile akıllı yanıtlar üretir' }
    ];
    
    showModal({
        title: t('sazan.title'),
        content: `
            <div class="sazan-modes">
                ${modes.map(mode => `
                    <div class="sazan-mode ${currentMode === mode.id ? 'active' : ''}" 
                         onclick="app.setSazanMode(${mode.id})">
                        <div class="sazan-mode__header">
                            <span class="sazan-mode__id">${mode.id}</span>
                            <span class="sazan-mode__name">${mode.name}</span>
                        </div>
                        <p class="sazan-mode__desc">${mode.desc}</p>
                    </div>
                `).join('')}
            </div>
        `,
        buttons: [
            { text: t('common.close'), class: 'btn--secondary', onClick: 'closeModal(this)' }
        ]
    });
}

// Toast Notification Component
function showToast(message, type = 'info', duration = 3000) {
    const container = document.getElementById('toastContainer');
    if (!container) return;

    const toast = document.createElement('div');
    toast.className = `toast toast--${type}`;
    toast.innerHTML = `
        <span class="toast__icon">${type === 'success' ? '✓' : type === 'error' ? '✕' : 'ℹ'}</span>
        <span class="toast__message">${message}</span>
    `;

    container.appendChild(toast);

    setTimeout(() => {
        toast.style.animation = 'slideOut 0.3s ease forwards';
        setTimeout(() => toast.remove(), 300);
    }, duration);
}

// Modal Component
function showModal(options) {
    const { title, content, buttons = [], onClose } = options;

    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';
    overlay.innerHTML = `
        <div class="modal">
            <div class="modal__header">
                <h3 class="modal__title">${title}</h3>
                <button class="modal__close" onclick="closeModal(this)">×</button>
            </div>
            <div class="modal__body">${content}</div>
            ${buttons.length ? `
                <div class="modal__footer">
                    ${buttons.map(btn => `
                        <button class="btn ${btn.class || 'btn--secondary'}" 
                                onclick="${btn.onClick}">
                            ${btn.text}
                        </button>
                    `).join('')}
                </div>
            ` : ''}
        </div>
    `;

    document.body.appendChild(overlay);
    requestAnimationFrame(() => overlay.classList.add('open'));

    const close = () => {
        overlay.classList.remove('open');
        setTimeout(() => overlay.remove(), 300);
        if (onClose) onClose();
    };

    overlay.addEventListener('click', (e) => {
        if (e.target === overlay) close();
    });

    window.closeModal = (btn) => {
        const overlay = btn.closest('.modal-overlay');
        if (overlay) {
            overlay.classList.remove('open');
            setTimeout(() => overlay.remove(), 300);
        }
    };

    return { close };
}

// Export
window.Component = Component;
window.DashboardPage = DashboardPage;
window.LoginPage = LoginPage;
window.showToast = showToast;
window.showModal = showModal;
window.showSazanModal = showSazanModal;
export { Component, DashboardPage, LoginPage, showToast, showModal, showSazanModal };
