/**
 * Fırat Shadow Handbook - Reactive State Management
 * Zero dependency - Observer pattern
 */

class Store {
    constructor(initialState = {}) {
        this._state = initialState;
        this._listeners = new Map();
        this._middlewares = [];
    }

    // Get current state
    get state() {
        return { ...this._state };
    }

    // Get specific value
    get(key) {
        return key ? this._state[key] : this._state;
    }

    // Set value and notify listeners
    set(key, value) {
        const oldValue = this._state[key];
        
        // Run middlewares
        for (const middleware of this._middlewares) {
            const result = middleware(key, value, oldValue);
            if (result === false) return; // Cancel update
            if (result !== undefined) value = result;
        }

        this._state[key] = value;
        
        // Notify listeners
        if (this._listeners.has(key)) {
            for (const listener of this._listeners.get(key)) {
                listener(value, oldValue, key);
            }
        }
        
        // Notify global listeners
        if (this._listeners.has('*')) {
            for (const listener of this._listeners.get('*')) {
                listener(this._state, key);
            }
        }
    }

    // Update multiple values
    update(updates) {
        for (const [key, value] of Object.entries(updates)) {
            this.set(key, value);
        }
    }

    // Subscribe to changes
    subscribe(key, listener) {
        if (!this._listeners.has(key)) {
            this._listeners.set(key, new Set());
        }
        this._listeners.get(key).add(listener);
        
        // Return unsubscribe function
        return () => {
            this._listeners.get(key).delete(listener);
        };
    }

    // Add middleware
    use(middleware) {
        this._middlewares.push(middleware);
    }

    // Persist to localStorage
    persist(keys = null) {
        const data = keys 
            ? Object.fromEntries(keys.map(k => [k, this._state[k]]))
            : this._state;
        localStorage.setItem('app_state', JSON.stringify(data));
    }

    // Restore from localStorage
    restore() {
        try {
            const saved = localStorage.getItem('app_state');
            if (saved) {
                const data = JSON.parse(saved);
                this._state = { ...this._state, ...data };
            }
        } catch (e) {
            console.warn('Failed to restore state:', e);
        }
    }
}

// Global app store
const store = new Store({
    // User
    user: null,
    isAuthenticated: false,
    role: null, // 'student' | 'teacher' | null
    
    // UI
    theme: 'dark',
    sidebarOpen: false,
    currentPage: '/',
    loading: false,
    
    // Data
    courses: [],
    recordings: [],
    calendar: [],
    messages: [],
    
    // Settings
    language: 'tr',
    notifications: true,
    
    // Sazan.avi (student only)
    sazanMode: 0, // 0: off, 1: manual, 2: semi-auto, 3: full-auto, 4: AI
    
    // Recording (teacher only)
    isRecording: false,
    recordingSettings: {
        quality: '1080p',
        autoRecord: true
    }
});

// Middleware: Log state changes in dev
if (location.hostname === 'localhost' || location.hostname === '127.0.0.1') {
    store.use((key, newValue, oldValue) => {
        console.log(`[Store] ${key}:`, oldValue, '→', newValue);
    });
}

// Middleware: Persist theme and language
store.use((key, value) => {
    if (key === 'theme' || key === 'language') {
        localStorage.setItem(`app_${key}`, value);
    }
});

// Restore persisted state on load
store.restore();

// Export
window.store = store;
export { Store, store };
