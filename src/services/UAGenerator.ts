/**
 * UAGenerator - Safe User-Agent Generator
 * 
 * Generates consistent, fingerprint-safe User-Agent strings.
 * CRITICAL: Only masks within the SAME OS family to avoid WAF detection.
 */

import { Platform } from 'react-native';
import DeviceInfo from 'react-native-device-info';
import SecureStorage from './SecureStorage';

const UA_STORAGE_KEY = 'pinned_user_agent';

// iOS Device Pool (iPhone models commonly used by students)
const IOS_DEVICES = [
    { model: 'iPhone14,5', name: 'iPhone 13' },
    { model: 'iPhone14,2', name: 'iPhone 13 Pro' },
    { model: 'iPhone15,2', name: 'iPhone 14 Pro' },
    { model: 'iPhone15,3', name: 'iPhone 14 Pro Max' },
    { model: 'iPhone16,1', name: 'iPhone 15 Pro' },
];

// Android Device Pool (Popular mid-range to flagship devices)
const ANDROID_DEVICES = [
    { manufacturer: 'Samsung', model: 'SM-G991B', name: 'Galaxy S21' },
    { manufacturer: 'Samsung', model: 'SM-S908B', name: 'Galaxy S22 Ultra' },
    { manufacturer: 'Xiaomi', model: 'M2102K1G', name: 'Redmi Note 10 Pro' },
    { manufacturer: 'OnePlus', model: 'LE2115', name: 'OnePlus 9 Pro' },
];

// Safari versions tied to iOS versions
const IOS_SAFARI_VERSIONS: Record<string, string> = {
    '17': '17.0',
    '16': '16.6',
    '15': '15.6.1',
};

// Chrome versions for Android
const ANDROID_CHROME_VERSIONS = ['120.0.0.0', '119.0.0.0', '118.0.0.0'];

interface GeneratedUA {
    userAgent: string;
    platform: 'ios' | 'android';
    deviceModel: string;
    osVersion: string;
}

/**
 * Generates a randomized but consistent User-Agent
 */
async function generateUserAgent(): Promise<GeneratedUA> {
    const osVersion = DeviceInfo.getSystemVersion();
    const majorVersion = osVersion.split('.')[0];

    if (Platform.OS === 'ios') {
        // iOS: Generate Safari UA
        const device = IOS_DEVICES[Math.floor(Math.random() * IOS_DEVICES.length)];
        const safariVersion = IOS_SAFARI_VERSIONS[majorVersion] || '17.0';
        const webKitVersion = '605.1.15';

        const ua = `Mozilla/5.0 (iPhone; CPU iPhone OS ${osVersion.replace(/\./g, '_')} like Mac OS X) AppleWebKit/${webKitVersion} (KHTML, like Gecko) Version/${safariVersion} Mobile/15E148 Safari/${webKitVersion}`;

        return {
            userAgent: ua,
            platform: 'ios',
            deviceModel: device.name,
            osVersion,
        };
    } else {
        // Android: Generate Chrome UA
        const device = ANDROID_DEVICES[Math.floor(Math.random() * ANDROID_DEVICES.length)];
        const chromeVersion = ANDROID_CHROME_VERSIONS[Math.floor(Math.random() * ANDROID_CHROME_VERSIONS.length)];

        const ua = `Mozilla/5.0 (Linux; Android ${osVersion}; ${device.model}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/${chromeVersion} Mobile Safari/537.36`;

        return {
            userAgent: ua,
            platform: 'android',
            deviceModel: device.name,
            osVersion,
        };
    }
}

/**
 * Gets or creates a pinned User-Agent
 * 
 * IMPORTANT: Once generated, the UA is pinned to prevent fingerprint inconsistency.
 * Clear storage to regenerate.
 */
export async function getPinnedUserAgent(): Promise<GeneratedUA> {
    const stored = await SecureStorage.get(UA_STORAGE_KEY);

    if (stored) {
        try {
            return JSON.parse(stored) as GeneratedUA;
        } catch {
            // Corrupted data, regenerate
        }
    }

    const newUA = await generateUserAgent();
    await SecureStorage.set(UA_STORAGE_KEY, JSON.stringify(newUA));

    return newUA;
}

/**
 * Force regenerate User-Agent (use sparingly)
 */
export async function regenerateUserAgent(): Promise<GeneratedUA> {
    await SecureStorage.delete(UA_STORAGE_KEY);
    return getPinnedUserAgent();
}

/**
 * Get simple UA string for WebView injection
 */
export async function getUserAgentString(): Promise<string> {
    const ua = await getPinnedUserAgent();
    return ua.userAgent;
}

export default {
    getPinnedUserAgent,
    regenerateUserAgent,
    getUserAgentString,
};
