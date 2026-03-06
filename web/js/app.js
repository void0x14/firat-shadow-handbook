/**
 * Fırat Shadow Handbook - Main Application
 * Entry point - Initializes all modules
 *
 * Security: XSS Prevention - All user-generated content must be escaped
 */

// Security: HTML escaping utility to prevent XSS
function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// App initialization
class App {
    constructor() {
        this.store = store;
        this.router = router;
        this.i18n = i18n;
        this.pages = {};
        this.lastLoginError = '';
    }

    async init() {
        console.log('🚀 Fırat Shadow Handbook initializing...');

        // Check for CAS callback errors in URL
        this.handleCasErrors();

        // Restore session (cookie-based)
        await this.restoreSession();

        // Load translations and set locale
        await this.i18n.setLocale(this.store.get('language'));

        // Setup routes
        this.setupRoutes();

        // Setup event listeners
        this.setupEventListeners();

        // Apply saved theme
        this.applyTheme(this.store.get('theme'));

        console.log('✅ App initialized');
    }

    handleCasErrors() {
        const hash = window.location.hash; // e.g. #/login?error=invalid_ticket
        const errorMatch = hash.match(/[?&]error=([^&]+)/);
        if (errorMatch) {
            const errorCode = errorMatch[1];
            const messages = {
                'no_ticket': 'CAS\'tan ticket alınamadı.',
                'invalid_ticket': 'Ticket doğrulanamadı. Lütfen tekrar deneyin.',
                'cas_error': 'CAS sunucusuna bağlanılamadı.'
            };
            const msg = messages[errorCode] || `Giriş hatası: ${errorCode}`;
            // Show error after DOM ready
            setTimeout(() => showToast(msg, 'error', 5000), 500);
            // Clean error from URL
            window.location.hash = '#/login';
        }
    }

    setupRoutes() {
        this.router
            .on('/', () => this.renderPage('dashboard'))
            .on('/courses', () => this.renderPage('courses'))
            .on('/courses/:id', (params) => this.renderPage('course-detail', params))
            .on('/calendar', () => this.renderPage('calendar'))
            .on('/recordings', () => this.renderPage('recordings'))
            .on('/recordings/:id', (params) => this.renderPage('player', params))
            .on('/chat', () => this.renderPage('chat'))
            .on('/settings', () => this.renderPage('settings'))
            .on('/profile', () => this.renderPage('profile'))
            .on('/login', () => this.renderPage('login'))
            .beforeEach((to, from) => {
                // Show loading
                this.showLoading(true);

                // Check auth for protected routes
                const publicRoutes = ['/', '/login'];
                if (!publicRoutes.includes(to) && !this.store.get('isAuthenticated')) {
                    return '/login';
                }
            })
            .afterEach((path) => {
                // Hide loading
                this.showLoading(false);

                // Update sidebar active state
                this.updateSidebarActive(path);

                // Update store
                this.store.set('currentPage', path);
            });

        // Initialize router after routes are set up
        this.router.init();
    }

    setupEventListeners() {
        // Theme toggle
        document.getElementById('themeToggle')?.addEventListener('click', () => {
            this.toggleTheme();
        });

        // Menu toggle (mobile)
        document.getElementById('menuToggle')?.addEventListener('click', () => {
            this.toggleSidebar();
        });

        // Logout button - with confirmation
        const logoutBtn = document.getElementById('logoutBtn');
        if (logoutBtn) {
            logoutBtn.addEventListener('click', (e) => {
                e.preventDefault();
                e.stopPropagation();
                this.handleLogoutClick();
            });
        }

        // Close sidebar on overlay click
        document.addEventListener('click', (e) => {
            if (e.target.classList.contains('sidebar-overlay')) {
                this.toggleSidebar(false);
            }
        });

        // Language change
        this.store.subscribe('language', async (lang) => {
            await this.i18n.setLocale(lang);
        });

        // User role change - update UI
        this.store.subscribe('role', (role) => {
            this.updateRoleBasedUI(role);
        });
    }

    handleLogoutClick() {
        // Show confirmation dialog
        const confirmed = confirm('Çıkış yapmak istediğinize emin misiniz?');
        if (confirmed) {
            this.logout();
        }
    }

    renderPage(name, params = {}) {
        const content = document.getElementById('content');
        if (!content) return;

        switch (name) {
            case 'dashboard':
                const dashboard = new DashboardPage(content);
                dashboard.render();
                break;

            case 'login':
                const loginPage = new LoginPage(content);
                loginPage.render();
                break;

            case 'courses':
                // Security: Use safe HTML rendering
                content.innerHTML = this.renderCoursesPage();
                break;

            case 'course-detail':
                // Security: Escape dynamic parameter
                const escapedId = escapeHtml(params.id.toString());
                content.innerHTML = this.renderCourseDetailPage(escapedId);
                break;

            case 'recordings':
                content.innerHTML = this.renderRecordingsPage();
                break;

            case 'player':
                // Security: Escape dynamic parameter
                const escapedPlayerId = escapeHtml(params.id.toString());
                content.innerHTML = this.renderPlayerPage(escapedPlayerId);
                break;

            case 'settings':
                content.innerHTML = this.renderSettingsPage();
                break;

            case 'profile':
                content.innerHTML = this.renderProfilePage();
                break;

            default:
                content.innerHTML = `
                    <div class="empty-state">
                        <div class="empty-state__icon">🚧</div>
                        <h2 class="empty-state__title">${t('common.comingSoon')}</h2>
                        <p class="empty-state__text">${t('common.pageInDevelopment')}</p>
                    </div>
                `;
        }

        // Update i18n after rendering
        this.i18n.updateAll();
    }

    // Theme management
    toggleTheme() {
        const current = this.store.get('theme');
        const next = current === 'dark' ? 'light' : 'dark';
        this.applyTheme(next);
        this.store.set('theme', next);
    }

    applyTheme(theme) {
        document.documentElement.setAttribute('data-theme', theme);
    }

    // Sidebar management
    toggleSidebar(open = null) {
        const sidebar = document.getElementById('sidebar');
        const isOpen = open ?? !sidebar.classList.contains('open');
        sidebar.classList.toggle('open', isOpen);
        this.store.set('sidebarOpen', isOpen);
    }

    updateSidebarActive(path) {
        document.querySelectorAll('.sidebar__item').forEach(item => {
            const route = item.getAttribute('data-nav');
            item.classList.toggle('active', route === path);
        });
    }

    // Role-based UI
    updateRoleBasedUI(role) {
        // Show/hide Sazan button
        document.querySelectorAll('.btn--sazan').forEach(btn => {
            btn.style.display = role === 'student' ? 'inline-flex' : 'none';
        });

        // Show/hide teacher-only elements
        document.querySelectorAll('[data-role="teacher"]').forEach(el => {
            el.style.display = role === 'teacher' ? '' : 'none';
        });

        // Show/hide student-only elements
        document.querySelectorAll('[data-role="student"]').forEach(el => {
            el.style.display = role === 'student' ? '' : 'none';
        });
    }

    // Loading state
    showLoading(show) {
        const loading = document.getElementById('loading');
        if (loading) {
            loading.classList.toggle('loading--hidden', !show);
        }
    }

    // Session management - uses real API
    async restoreSession() {
        const maxAttempts = 2;
        for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
            try {
                const response = await fetch('/api/validate-session', {
                    method: 'GET',
                    credentials: 'include'
                });

                if (response.ok) {
                    const data = await response.json();
                    if (data.valid) {
                        const user = {
                            id: 1,
                            name: data.full_name || data.user,
                            username: data.user,
                            email: data.email || '',
                            role: 'student'
                        };
                        this.store.update({
                            user,
                            isAuthenticated: true,
                            role: user.role
                        });
                        this.updateUserName(user.name);
                        return true;
                    }
                }

                if (attempt < maxAttempts && (response.status === 401 || response.status === 403 || response.status === 429 || response.status >= 500)) {
                    await new Promise(resolve => setTimeout(resolve, 200));
                    continue;
                }
            } catch (error) {
                if (attempt < maxAttempts) {
                    await new Promise(resolve => setTimeout(resolve, 200));
                    continue;
                }
                console.error('Session restore failed:', error);
            }
            break;
        }

        // Clear invalid session
        this.store.update({
            user: null,
            isAuthenticated: false,
            role: null
        });
        localStorage.removeItem('app_user');
        return false;
    }

    getCookie(name) {
        const match = document.cookie.match(new RegExp('(^| )' + name + '=([^;]+)'));
        return match ? decodeURIComponent(match[2]) : null;
    }

    updateUserName(name) {
        const nameEl = document.getElementById('userName');
        const roleEl = document.getElementById('userRole');
        if (nameEl) nameEl.textContent = name;
        if (roleEl) roleEl.textContent = t(`role.${this.store.get('role')}`);
    }

    // Real login using API
    async login(username, password) {
        try {
            this.lastLoginError = '';
            const response = await fetch('/api/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                credentials: 'include', // Include cookies
                body: JSON.stringify({ username, password })
            });

            const data = await response.json();

            if (data.success) {
                const user = {
                    id: 1,
                    name: data.full_name || data.user,
                    username: data.user,
                    email: data.email || '',
                    role: 'student'
                };
                this.store.update({
                    user,
                    isAuthenticated: true,
                    role: user.role
                });
                this.updateUserName(user.name);
                showToast(t('login.success'), 'success');
                this.router.navigate('/');
                return true;
            } else {
                this.lastLoginError = data.error || t('login.failed');
                showToast(this.lastLoginError, 'error');
                return false;
            }
        } catch (error) {
            console.error('Login failed:', error);
            this.lastLoginError = t('login.error');
            showToast(this.lastLoginError, 'error');
            return false;
        }
    }

    renderCoursesPage() {
        return `<h1>${t('nav.courses')}</h1><p>${t('common.comingSoon')}</p>`;
    }

    renderCourseDetailPage(id) {
        return `<h1>${t('courses.detail')} #${id}</h1><p>${t('common.comingSoon')}</p>`;
    }

    renderRecordingsPage() {
        return `<h1>${t('nav.recordings')}</h1><p>${t('common.comingSoon')}</p>`;
    }

    renderPlayerPage(id) {
        return `<h1>${t('player.title')} #${id}</h1><p>${t('common.comingSoon')}</p>`;
    }

    renderSettingsPage() {
        return `
            <div class="page-header">
                <h1 class="page-header__title">${t('nav.settings')}</h1>
            </div>
            <div class="settings-page">
                <section class="settings-section">
                    <h3 class="settings-section__title">${t('settings.language')}</h3>
                    <div class="settings-row">
                        <span class="settings-row__label">Dil seçimi</span>
                        <select onchange="app.changeLanguage(this.value)">
                            <option value="tr" ${this.store.get('language') === 'tr' ? 'selected' : ''}>Türkçe</option>
                            <option value="en" ${this.store.get('language') === 'en' ? 'selected' : ''}>English</option>
                        </select>
                    </div>
                </section>
                <section class="settings-section">
                    <h3 class="settings-section__title">${t('settings.theme')}</h3>
                    <div class="settings-row">
                        <span class="settings-row__label">Görünüm</span>
                        <select onchange="app.applyTheme(this.value); store.set('theme', this.value)">
                            <option value="dark" ${this.store.get('theme') === 'dark' ? 'selected' : ''}>${t('settings.darkTheme')}</option>
                            <option value="light" ${this.store.get('theme') === 'light' ? 'selected' : ''}>${t('settings.lightTheme')}</option>
                        </select>
                    </div>
                </section>
            </div>
        `;
    }

    renderProfilePage() {
        const user = this.store.get('user');
        return `
            <h1>${t('nav.profile')}</h1>
            <div class="profile-page">
                <div class="card">
                    <div class="profile-header">
                        <div class="avatar avatar--xl">${user?.name?.[0] || '?'}</div>
                        <h2>${user?.name || t('user.guest')}</h2>
                        <p>${t(`role.${this.store.get('role')}`)}</p>
                    </div>
                </div>
            </div>
        `;
    }

    // Actions

    // Real logout using API
    async logout() {
        try {
            await fetch('/api/logout', {
                method: 'POST',
                credentials: 'include'
            });
        } catch (error) {
            console.error('Logout API call failed:', error);
        }

        // Clear ShadowUser cookie client-side
        document.cookie = 'ShadowUser=; Path=/; Max-Age=0';

        // Clear local state regardless of API result
        this.store.update({
            user: null,
            isAuthenticated: false,
            role: null
        });
        localStorage.removeItem('app_user');
        this.router.navigate('/login');
        showToast('Çıkış yapıldı', 'success');
    }

    async changeLanguage(lang) {
        this.store.set('language', lang);
        await this.i18n.setLocale(lang);
        // Re-render current page
        this.router.navigate();
        showToast(t('settings.languageChanged'), 'success');
    }

    joinLive(id) {
        showToast('Canlı ders özelliği yakında aktif olacak!', 'info');
    }

    showSazanModal() {
        window.showSazanModal();
    }

    setSazanMode(mode) {
        this.store.set('sazanMode', mode);
        window.closeModal(document.querySelector('.modal__close'));
        showToast(`${t('sazan.mode')}: ${mode}`, 'success');
    }

    startClass() {
        showToast(t('teacher.startingClass'), 'info');
        // Teacher-only action
    }

    toggleRecording() {
        const isRecording = this.store.get('isRecording');
        this.store.set('isRecording', !isRecording);
        showToast(isRecording ? t('recording.stopped') : t('recording.started'), 'info');
    }
}

// Initialize app
const app = new App();
document.addEventListener('DOMContentLoaded', () => app.init());

// Export
window.app = app;
export { App, app };
