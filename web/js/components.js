/**
 * Fırat Shadow Handbook - UI Components
 * Professional, minimal UI - Zero dependency
 */

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
        const courses = window.MOCK_DATA.courses;

        return `
            <div class="dashboard">
                <!-- Page Header -->
                <div class="page-header">
                    <div class="page-header__content">
                        <h1 class="page-header__title">${t('dashboard.welcome')}, ${user?.name || t('user.guest')}</h1>
                        <p class="page-header__subtitle">${t('dashboard.subtitle')}</p>
                    </div>
                </div>

                <!-- Live Classes Section -->
                <section class="section section--live">
                    <div class="section__header">
                        <h2 class="section__title">${t('dashboard.liveClasses')}</h2>
                        <div class="section__badge">
                            <span class="live-indicator"></span>
                            <span>${window.MOCK_DATA.liveClasses.filter(c => c.status === 'live').length} ${t('live.active')}</span>
                        </div>
                    </div>
                    <div class="live-grid">
                        ${this.renderLiveClasses()}
                    </div>
                </section>

                <!-- Two Column Layout -->
                <div class="dashboard-columns">
                    <!-- Courses Section -->
                    <section class="section section--courses">
                        <div class="section__header">
                            <h2 class="section__title">${t('dashboard.myCourses')}</h2>
                            <a href="#/courses" class="link-arrow">${t('common.viewAll')}</a>
                        </div>
                        <div class="courses-grid">
                            ${this.renderCourses(courses)}
                        </div>
                    </section>

                    <!-- Sidebar Widgets -->
                    <aside class="sidebar-widgets">
                        ${this.renderQuickActions(role)}
                        ${this.renderRecentRecordings()}
                    </aside>
                </div>
            </div>
        `;
    }

    renderLiveClasses() {
        return window.MOCK_DATA.liveClasses.map(cls => `
            <div class="live-card ${cls.status === 'live' ? 'live-card--active' : ''}">
                <div class="live-card__top">
                    <div class="live-card__status">
                        ${cls.status === 'live' 
                            ? `<span class="status-badge status-badge--live">${t('live.now')}</span>` 
                            : `<span class="status-badge status-badge--upcoming">${t('live.upcoming')}</span>`
                        }
                    </div>
                    <div class="live-card__time">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="10"></circle>
                            <polyline points="12 6 12 12 16 14"></polyline>
                        </svg>
                        <span>${cls.time}</span>
                    </div>
                </div>
                <div class="live-card__body">
                    <h3 class="live-card__title">${cls.name}</h3>
                    <p class="live-card__instructor">${cls.instructor}</p>
                    <div class="live-card__meta">
                        <span class="live-card__participants">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
                                <circle cx="9" cy="7" r="4"></circle>
                                <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
                                <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
                            </svg>
                            ${cls.participants}/${cls.capacity} katılımcı
                        </span>
                    </div>
                </div>
                <div class="live-card__actions">
                    <button class="btn btn--primary btn--sm" onclick="app.joinLive(${cls.id})">
                        ${t('live.join')}
                    </button>
                    ${store.get('role') === 'student' ? `
                        <button class="btn btn--outline btn--sm" onclick="app.showSazanModal()">
                            Sazan.avi
                        </button>
                    ` : ''}
                </div>
            </div>
        `).join('');
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

    renderRecentRecordings() {
        return `
            <div class="widget">
                <h3 class="widget__title">${t('recordings.recent')}</h3>
                <div class="recordings-list">
                    ${window.MOCK_DATA.recordings.slice(0, 3).map(rec => `
                        <a href="#/recordings/${rec.id}" class="recording-item ${rec.watched ? 'recording-item--watched' : ''}">
                            <div class="recording-item__play">
                                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                                    <polygon points="5 3 19 12 5 21 5 3"></polygon>
                                </svg>
                            </div>
                            <div class="recording-item__info">
                                <span class="recording-item__course">${rec.course}</span>
                                <span class="recording-item__meta">${rec.date} · ${rec.duration}</span>
                            </div>
                        </a>
                    `).join('')}
                </div>
            </div>
        `;
    }

    renderCourses(courses) {
        return courses.map(course => `
            <a href="#/courses/${course.id}" class="course-card">
                <div class="course-card__header">
                    <span class="course-card__category">${course.category}</span>
                    <span class="course-card__code">${course.code}</span>
                </div>
                <h3 class="course-card__title">${course.name}</h3>
                <div class="course-card__stats">
                    <span>${course.recordings} kayıt</span>
                </div>
                <div class="course-card__progress">
                    <div class="progress-bar">
                        <div class="progress-bar__fill" style="width: ${course.progress}%"></div>
                    </div>
                    <span class="progress-label">${course.progress}%</span>
                </div>
            </a>
        `).join('');
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
                        
                        <a href="https://debsis.firat.edu.tr" class="login-btn login-btn--primary">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"></path>
                                <polyline points="10 17 15 12 10 7"></polyline>
                                <line x1="15" y1="12" x2="3" y2="12"></line>
                            </svg>
                            <span>DEBSİS ile Giriş Yap</span>
                        </a>
                        
                        <div class="login-divider">
                            <span>veya</span>
                        </div>
                        
                        <button class="login-btn login-btn--secondary" onclick="app.demoLogin()">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
                                <circle cx="12" cy="7" r="4"></circle>
                            </svg>
                            <span>Demo Giriş</span>
                        </button>
                        
                        <p class="login-note">
                            DEBSİS'e giriş yaptıktan sonra otomatik olarak yönlendirileceksiniz.
                        </p>
                    </div>
                </div>
            </div>
        `;
    }
}

// Sazan.avi Modal
function showSazanModal() {
    const currentMode = store.get('sazanMode');
    const modes = window.MOCK_DATA.sazanModes;
    
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
