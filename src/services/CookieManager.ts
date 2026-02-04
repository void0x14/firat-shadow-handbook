/**
 * CookieManager - ASP.NET Session Cookie Handler
 * 
 * Extracts and persists session cookies from WebView.
 * Critical for maintaining login state across app restarts.
 */

import CookieManager from '@react-native-cookies/cookies';
import SecureStorage from './SecureStorage';

const COOKIE_STORAGE_KEY = 'obs_session_cookies';
const OBS_DOMAIN = 'obs.firat.edu.tr';

export interface SessionCookies {
    aspNetSessionId: string | null;
    authCookie: string | null;
    extractedAt: number;
}

/**
 * Extract cookies from the OBS domain
 */
export async function extractOBSCookies(): Promise<SessionCookies> {
    try {
        const cookies = await CookieManager.get(`https://${OBS_DOMAIN}`);

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
    await CookieManager.clearAll();
}

/**
 * Inject stored cookies back into WebView
 */
export async function injectStoredCookies(): Promise<boolean> {
    const cookies = await getStoredCookies();
    if (!cookies || !cookies.aspNetSessionId) return false;

    try {
        await CookieManager.set(`https://${OBS_DOMAIN}`, {
            name: 'ASP.NET_SessionId',
            value: cookies.aspNetSessionId,
            domain: OBS_DOMAIN,
            path: '/',
            secure: true,
            httpOnly: true,
        });

        if (cookies.authCookie) {
            await CookieManager.set(`https://${OBS_DOMAIN}`, {
                name: '.ASPXAUTH',
                value: cookies.authCookie,
                domain: OBS_DOMAIN,
                path: '/',
                secure: true,
                httpOnly: true,
            });
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
