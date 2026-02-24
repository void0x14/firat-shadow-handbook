/**
 * Fırat Shadow Handbook - Internationalization (i18n)
 * Zero dependency - Pure JavaScript
 */

class I18n {
    constructor(options = {}) {
        this.locale = options.locale || 'tr';
        this.fallbackLocale = options.fallbackLocale || 'en';
        this.translations = {};
        this.listeners = new Set();
    }

    // Load translations
    async load(locale) {
        try {
            const response = await fetch(`/i18n/${locale}.json`);
            if (!response.ok) throw new Error(`Failed to load ${locale}`);
            this.translations[locale] = await response.json();
            return true;
        } catch (error) {
            console.error(`[i18n] Failed to load ${locale}:`, error);
            if (locale !== this.fallbackLocale) {
                return this.load(this.fallbackLocale);
            }
            return false;
        }
    }

    // Set current locale
    async setLocale(locale) {
        if (!this.translations[locale]) {
            await this.load(locale);
        }
        this.locale = locale;
        document.documentElement.lang = locale;
        this.updateAll();
        this.listeners.forEach(fn => fn(locale));
    }

    // Get translation
    t(key, params = {}) {
        const keys = key.split('.');
        let value = this.translations[this.locale];
        
        for (const k of keys) {
            if (!value || typeof value !== 'object') {
                // Try fallback
                value = this.translations[this.fallbackLocale];
                for (const fk of keys) {
                    if (!value || typeof value !== 'object') return key;
                    value = value[fk];
                }
                break;
            }
            value = value[k];
        }
        
        if (typeof value !== 'string') return key;
        
        // Replace parameters {{param}}
        return value.replace(/\{\{(\w+)\}\}/g, (_, name) => {
            return params[name] !== undefined ? params[name] : `{{${name}}}`;
        });
    }

    // Update all elements with data-i18n attribute
    updateAll() {
        document.querySelectorAll('[data-i18n]').forEach(el => {
            const key = el.getAttribute('data-i18n');
            const params = el.getAttribute('data-i18n-params');
            const parsedParams = params ? JSON.parse(params) : {};
            el.textContent = this.t(key, parsedParams);
        });

        document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
            const key = el.getAttribute('data-i18n-placeholder');
            el.placeholder = this.t(key);
        });

        document.querySelectorAll('[data-i18n-title]').forEach(el => {
            const key = el.getAttribute('data-i18n-title');
            el.title = this.t(key);
        });
    }

    // Subscribe to locale changes
    subscribe(fn) {
        this.listeners.add(fn);
        return () => this.listeners.delete(fn);
    }
}

// Initialize i18n
const i18n = new I18n({
    locale: localStorage.getItem('app_language') || navigator.language.split('-')[0] || 'tr',
    fallbackLocale: 'en'
});

// Auto-load and apply
document.addEventListener('DOMContentLoaded', async () => {
    await i18n.load(i18n.locale);
    i18n.updateAll();
});

// Export
window.i18n = i18n;
window.t = (key, params) => i18n.t(key, params);
export { I18n, i18n };
