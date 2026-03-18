/**
 * Fırat Shadow Handbook - SPA Router
 * Zero dependency - Hash-based routing
 */

class Router {
    constructor(routes = {}) {
        this.routes = routes;
        this.currentRoute = null;
        this.params = {};
        this.beforeHooks = [];
        this.afterHooks = [];
        this.initialized = false;
        
        // Listen for hash changes (but don't navigate on construction)
        window.addEventListener('hashchange', () => {
            if (this.initialized) this.navigate();
        });
    }

    // Initialize router - call this after routes are set up
    init() {
        this.initialized = true;
        // Force navigate to root on init
        this.navigate('/');
    }

    // Add route
    on(path, handler) {
        this.routes[path] = handler;
        return this;
    }

    // Navigate to path
    navigate(path = null) {
        if (path) {
            const newHash = '#' + path;
            // Only change hash if different
            if (window.location.hash !== newHash) {
                window.location.hash = path;
                return;
            }
            // If same, continue to handler
        }

        // Parse current hash
        const hash = window.location.hash.slice(1) || '/';
        const [pathPart, queryString] = hash.split('?');
        
        // Parse query params
        const query = {};
        if (queryString) {
            queryString.split('&').forEach(pair => {
                const [key, value] = pair.split('=');
                query[decodeURIComponent(key)] = decodeURIComponent(value || '');
            });
        }

        // Find matching route
        let handler = null;
        let params = {};

        for (const [pattern, fn] of Object.entries(this.routes)) {
            const match = this.matchPattern(pattern, pathPart);
            if (match) {
                handler = fn;
                params = match;
                break;
            }
        }

        // Run before hooks
        for (const hook of this.beforeHooks) {
            const result = hook(pathPart, this.currentRoute);
            if (result === false) return;
            if (typeof result === 'string') {
                window.location.hash = result;
                return;
            }
        }

        // Update state
        this.currentRoute = pathPart;
        this.params = { ...params, ...query };

        // Execute handler
        if (handler) {
            handler(this.params, query);
        } else {
            this.handle404(pathPart);
        }

        // Run after hooks
        for (const hook of this.afterHooks) {
            hook(pathPart);
        }
    }

    // Match pattern with path
    matchPattern(pattern, path) {
        const patternParts = pattern.split('/').filter(Boolean);
        const pathParts = path.split('/').filter(Boolean);

        if (patternParts.length !== pathParts.length) {
            return null;
        }

        const params = {};

        for (let i = 0; i < patternParts.length; i++) {
            const pp = patternParts[i];
            const actual = pathParts[i];

            if (pp.startsWith(':')) {
                params[pp.slice(1)] = actual;
            } else if (pp !== actual) {
                return null;
            }
        }

        return params;
    }

    // Before navigation hook
    beforeEach(hook) {
        this.beforeHooks.push(hook);
        return this;
    }

    // After navigation hook
    afterEach(hook) {
        this.afterHooks.push(hook);
        return this;
    }

    // 404 handler
    handle404(path) {
        console.warn(`[Router] 404: ${path}`);
        const content = document.getElementById('content');
        if (content) {
            content.innerHTML = `
                <div class="empty-state">
                    <div class="empty-state__icon">🔍</div>
                    <h2 class="empty-state__title">${t('error.pageNotFound')}</h2>
                    <p class="empty-state__text">${t('error.pageNotFoundDesc')}</p>
                    <a href="#/" class="btn btn--primary">${t('nav.goHome')}</a>
                </div>
            `;
        }
    }

    // Get current route info
    getRoute() {
        return {
            path: this.currentRoute,
            params: this.params
        };
    }
}

// Create router instance
const router = new Router();

// Export
window.router = router;
export { Router, router };
