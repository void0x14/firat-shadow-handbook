/**
 * CookieManager - ASP.NET Session Cookie Handler
 * 
 * Uses react-native-nitro-cookies (JSI-based, 5x faster than bridge)
 * for extracting and persisting session cookies from WebView.
 */

import NitroCookies from 'react-native-nitro-cookies';
import SecureStorage from './SecureStorage';

const COOKIE_STORAGE_KEY = 'obs_session_cookies';
const OBS_URL = 'https://obs.firat.edu.tr';

export interface SessionCookies {
    aspNetSessionId: string | null;
    authCookie: string | null;
    extractedAt: number;
}

/**
 * Extract cookies from the OBS domain
 * Uses WebKit cookie store on iOS for WebView sync
 */
export async function extractOBSCookies(): Promise<SessionCookies> {
    try {
        // Get cookies from WebKit store (synced with WebView)
        const useWebKit = true;
        const cookies = await NitroCookies.get(OBS_URL, useWebKit);

        const sessionCookies: SessionCookies = {
            aspNetSessionId: cookies['ASP.NET_SessionId']?.value || null,
            authCookie: cookies['.ASPXAUTH']?.value || null,
            extractedAt: Date.now(),
        };

        // Persist to secure storage
        await SecureStorage.set(COOKIE_STORAGE_KEY, JSON.stringify(sessionCookies));

        return sessionCookies;
    } catch (error) {
        console.error('[CookieManager] Failed to extract cookies:', error);
        throw error;
    }
}

/**
 * Get stored session cookies
 */
export async function getStoredCookies(): Promise<SessionCookies | null> {
    try {
        const stored = await SecureStorage.get(COOKIE_STORAGE_KEY);
        if (!stored) return null;

        return JSON.parse(stored) as SessionCookies;
    } catch {
        return null;
    }
}

/**
 * Check if we have a valid session
 */
export async function hasValidSession(): Promise<boolean> {
    const cookies = await getStoredCookies();
    if (!cookies) return false;

    // Session expires after 20 minutes of inactivity (ASP.NET default)
    const SESSION_TIMEOUT = 20 * 60 * 1000; // 20 minutes
    const isExpired = Date.now() - cookies.extractedAt > SESSION_TIMEOUT;

    return !isExpired && !!cookies.aspNetSessionId;
}

/**
 * Clear all stored cookies (logout)
 */
export async function clearCookies(): Promise<void> {
    await SecureStorage.delete(COOKIE_STORAGE_KEY);
    await NitroCookies.clearAll(true); // Clear WebKit cookies
    await NitroCookies.clearAll(false); // Clear native cookies
}

/**
 * Inject stored cookies back into WebView
 */
export async function injectStoredCookies(): Promise<boolean> {
    const cookies = await getStoredCookies();
    if (!cookies || !cookies.aspNetSessionId) return false;

    try {
        const useWebKit = true;

        await NitroCookies.set(OBS_URL, {
            name: 'ASP.NET_SessionId',
            value: cookies.aspNetSessionId,
            path: '/',
            secure: true,
            httpOnly: true,
        }, useWebKit);

        if (cookies.authCookie) {
            await NitroCookies.set(OBS_URL, {
                name: '.ASPXAUTH',
                value: cookies.authCookie,
                path: '/',
                secure: true,
                httpOnly: true,
            }, useWebKit);
        }

        return true;
    } catch (error) {
        console.error('[CookieManager] Failed to inject cookies:', error);
        return false;
    }
}

export default {
    extractOBSCookies,
    getStoredCookies,
    hasValidSession,
    clearCookies,
    injectStoredCookies,
};
