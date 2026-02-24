/**
 * Fırat Shadow Handbook - Main Application
 * Entry point - Initializes all modules
 */

// Mock Data
const MOCK_DATA = {
    liveClasses: [
        { id: 1, name: 'Internet Programcılığı I', instructor: 'Doç. Dr. Ahmet Yılmaz', time: '18:00 - 19:30', status: 'live', participants: 47, capacity: 60 },
        { id: 2, name: 'Veri Yapıları', instructor: 'Prof. Dr. Mehmet Kaya', time: '19:45 - 21:15', status: 'live', participants: 32, capacity: 45 },
        { id: 3, name: 'Makine Öğrenmesi', instructor: 'Dr. Elif Demir', time: '14:00 - 15:30', status: 'upcoming', participants: 0, capacity: 40 }
    ],
    courses: [
        { id: 1, name: 'Internet Programcılığı I', code: 'BIL311', category: '3. Sınıf', recordings: 12, progress: 45 },
        { id: 2, name: 'Veri Yapıları', code: 'BIL221', category: '2. Sınıf', recordings: 8, progress: 72 },
        { id: 3, name: 'Makine Öğrenmesi', code: 'BIL421', category: '4. Sınıf', recordings: 6, progress: 30 },
        { id: 4, name: 'Veritabanı Sistemleri', code: 'BIL301', category: '3. Sınıf', recordings: 10, progress: 88 }
    ],
    recordings: [
        { id: 1, course: 'Internet Programcılığı I', date: '2025-01-13', duration: '1:32:00', watched: true },
        { id: 2, course: 'Veri Yapıları', date: '2025-01-12', duration: '1:28:00', watched: false },
        { id: 3, course: 'Makine Öğrenmesi', date: '2025-01-11', duration: '1:45:00', watched: true }
    ],
    sazanModes: [
        { id: 0, name: 'Kapalı', desc: 'Sazan.avi devre dışı' },
        { id: 1, name: 'Manuel', desc: 'Sadece siz tetiklediğinde çalışır' },
        { id: 2, name: 'Yarı Otomatik', desc: 'Soruları algılar, onayınızı bekler' },
        { id: 3, name: 'Tam Otomatik', desc: 'Soruları algılar ve yanıtlar' },
        { id: 4, name: 'AI Modu', desc: 'LLM ile akıllı yanıtlar üretir' }
    ]
};
window.MOCK_DATA = MOCK_DATA;

// App initialization
class App {
    constructor() {
        this.store = store;
        this.router = router;
        this.i18n = i18n;
        this.pages = {};
    }

    async init() {
        console.log('🚀 Fırat Shadow Handbook initializing...');

        // Restore session FIRST (before router)
        await this.restoreSession();

        // Load translations and set locale
        await this.i18n.setLocale(this.store.get('language'));

        // Setup routes
        this.setupRoutes();

        // Setup event listeners
        this.setupEventListeners();

        // Apply saved theme
        this.applyTheme(this.store.get('theme'));

        // Restore session if exists
        await this.restoreSession();

        console.log('✅ App initialized');
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
                content.innerHTML = this.renderCoursesPage();
                break;

            case 'course-detail':
                content.innerHTML = this.renderCourseDetailPage(params.id);
                break;

            case 'recordings':
                content.innerHTML = this.renderRecordingsPage();
                break;

            case 'player':
                content.innerHTML = this.renderPlayerPage(params.id);
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
            const route = item.getAttribute('data-route');
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

    // Session management
    async restoreSession() {
        // Mock session for development
        const savedUser = localStorage.getItem('app_user');
        if (savedUser) {
            const user = JSON.parse(savedUser);
            this.store.update({
                user,
                isAuthenticated: true,
                role: user.role || 'student'
            });
        } else {
            // Mock user for development
            this.store.update({
                user: { name: 'Abdullah', id: 1, role: 'student' },
                isAuthenticated: true,
                role: 'student'
            });
        }
    }

    updateUserName(name) {
        const nameEl = document.getElementById('userName');
        const roleEl = document.getElementById('userRole');
        if (nameEl) nameEl.textContent = name;
        if (roleEl) roleEl.textContent = t(`role.${this.store.get('role')}`);
    }

    // Demo login
    demoLogin() {
        const mockUser = {
            id: 1,
            name: 'Abdullah',
            email: 'abdullah@firat.edu.tr',
            role: 'student'
        };
        this.store.update({
            user: mockUser,
            isAuthenticated: true,
            role: mockUser.role
        });
        localStorage.setItem('app_user', JSON.stringify(mockUser));
        this.updateUserName(mockUser.name);
        showToast('Giriş başarılı! Yönlendiriliyorsunuz...', 'success');
        setTimeout(() => {
            this.router.navigate('/');
        }, 500);
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
    async login() {
        // Redirect to CAS login
        showToast(t('login.redirecting'), 'info');
        // In real implementation, redirect to CAS
        // window.location.href = 'https://debsis.firat.edu.tr/login/cas.php';
        
        // For demo, simulate login
        setTimeout(() => {
            const mockUser = {
                id: 1,
                name: 'Abdullah',
                email: 'abdullah@example.com',
                role: 'student'
            };
            this.store.update({
                user: mockUser,
                isAuthenticated: true,
                role: mockUser.role
            });
            localStorage.setItem('app_user', JSON.stringify(mockUser));
            this.updateUserName(mockUser.name);
            showToast(t('login.success'), 'success');
            this.router.navigate('/');
        }, 1000);
    }

    logout() {
        this.store.update({
            user: null,
            isAuthenticated: false,
            role: null
        });
        localStorage.removeItem('app_user');
        this.router.navigate('/login');
        showToast(t('logout.success'), 'success');
    }

    async changeLanguage(lang) {
        this.store.set('language', lang);
        await this.i18n.setLocale(lang);
        // Re-render current page
        this.router.navigate();
        showToast(t('settings.languageChanged'), 'success');
    }

    joinLive(id) {
        const cls = window.MOCK_DATA.liveClasses.find(c => c.id === id);
        if (!cls) return;
        
        // Show loading toast
        const toast = document.createElement('div');
        toast.className = 'toast toast--loading';
        toast.innerHTML = `
            <div class="toast__spinner"></div>
            <span class="toast__message">${cls.name} dersine katılıyor...</span>
        `;
        document.getElementById('toastContainer').appendChild(toast);
        
        // Simulate connection
        setTimeout(() => {
            toast.remove();
            showToast(`${cls.name} dersine bağlandınız!`, 'success');
            // In real implementation: window.open(cls.joinUrl, '_blank');
        }, 1500);
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
