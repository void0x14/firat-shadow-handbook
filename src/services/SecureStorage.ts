/**
 * SecureStorage - Encrypted MMKV Wrapper
 * 
 * Uses MMKV for high-performance encrypted storage.
 * Encryption key is stored in native Keychain/Keystore via expo-secure-store.
 */

import { MMKV } from 'react-native-mmkv';
import * as SecureStore from 'expo-secure-store';

const ENCRYPTION_KEY_ID = 'shadow_vault_key';
const STORAGE_ID = 'shadow-vault';

let storage: MMKV | null = null;

/**
 * Generates or retrieves the encryption key from secure storage
 */
async function getOrCreateEncryptionKey(): Promise<string> {
    let key = await SecureStore.getItemAsync(ENCRYPTION_KEY_ID);

    if (!key) {
        // Generate a random 32-byte key
        const randomBytes = new Uint8Array(32);
        crypto.getRandomValues(randomBytes);
        key = Array.from(randomBytes)
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');

        await SecureStore.setItemAsync(ENCRYPTION_KEY_ID, key);
    }

    return key;
}

/**
 * Initialize the secure storage instance
 */
export async function initSecureStorage(): Promise<MMKV> {
    if (storage) return storage;

    const encryptionKey = await getOrCreateEncryptionKey();

    storage = new MMKV({
        id: STORAGE_ID,
        encryptionKey,
    });

    return storage;
}

/**
 * Get the storage instance (must be initialized first)
 */
export function getStorage(): MMKV {
    if (!storage) {
        throw new Error('SecureStorage not initialized. Call initSecureStorage() first.');
    }
    return storage;
}

// Convenience methods
export const SecureStorage = {
    async set(key: string, value: string): Promise<void> {
        const store = getStorage();
        store.set(key, value);
    },

    async get(key: string): Promise<string | undefined> {
        const store = getStorage();
        return store.getString(key);
    },

    async delete(key: string): Promise<void> {
        const store = getStorage();
        store.delete(key);
    },

    async getAllKeys(): Promise<string[]> {
        const store = getStorage();
        return store.getAllKeys();
    },

    async clear(): Promise<void> {
        const store = getStorage();
        store.clearAll();
    }
};

export default SecureStorage;
