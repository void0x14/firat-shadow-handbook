/**
 * LauncherScreen - Alpha Build Main Screen
 * 
 * Minimal UI for testing the Shadow Proxy login flow.
 */

import React, { useState, useCallback, useRef, useEffect } from 'react';
import {
    View,
    Text,
    TouchableOpacity,
    StyleSheet,
    SafeAreaView,
    StatusBar,
    ActivityIndicator,
} from 'react-native';
import HiddenWebView, { HiddenWebViewRef, WebViewLog } from '../components/HiddenWebView';
import VisualMonitor from '../components/VisualMonitor';
import { initSecureStorage } from '../services/SecureStorage';
import { hasValidSession, clearCookies } from '../services/CookieManager';

type LoginStatus = 'initializing' | 'not_logged_in' | 'logging_in' | 'logged_in';

const LauncherScreen: React.FC = () => {
    const [status, setStatus] = useState<LoginStatus>('initializing');
    const [showWebView, setShowWebView] = useState(false);
    const [showMonitor, setShowMonitor] = useState(__DEV__);
    const [logs, setLogs] = useState<WebViewLog[]>([]);

    const webViewRef = useRef<HiddenWebViewRef>(null);

    // Initialize storage and check session
    useEffect(() => {
        const init = async () => {
            try {
                await initSecureStorage();
                const hasSession = await hasValidSession();
                setStatus(hasSession ? 'logged_in' : 'not_logged_in');
            } catch (error) {
                console.error('Init failed:', error);
                setStatus('not_logged_in');
            }
        };
        init();
    }, []);

    const addLog = useCallback((log: WebViewLog) => {
        setLogs(prev => [log, ...prev].slice(0, 50)); // Keep last 50 logs
    }, []);

    const handleLogin = useCallback(() => {
        setStatus('logging_in');
        setShowWebView(true);
    }, []);

    const handleLoginSuccess = useCallback(() => {
        setStatus('logged_in');
        setShowWebView(false);
        addLog({
            timestamp: Date.now(),
            type: 'success',
            message: '🎉 Login complete! Session saved.',
        });
    }, [addLog]);

    const handleLogout = useCallback(async () => {
        await clearCookies();
        setStatus('not_logged_in');
        addLog({
            timestamp: Date.now(),
            type: 'info',
            message: 'Logged out. Session cleared.',
        });
    }, [addLog]);

    const getStatusInfo = () => {
        switch (status) {
            case 'initializing':
                return { text: 'Initializing...', color: '#a1a1aa', icon: '⏳' };
            case 'not_logged_in':
                return { text: 'Not Logged In', color: '#ef4444', icon: '🔒' };
            case 'logging_in':
                return { text: 'Logging In...', color: '#f59e0b', icon: '🔄' };
            case 'logged_in':
                return { text: 'Logged In', color: '#22c55e', icon: '✅' };
        }
    };

    const statusInfo = getStatusInfo();

    return (
        <SafeAreaView style={styles.container}>
            <StatusBar barStyle="light-content" backgroundColor="#0f0f23" />

            {/* Header */}
            <View style={styles.header}>
                <Text style={styles.title}>🌑 Shadow Handbook</Text>
                <Text style={styles.subtitle}>Fırat OBS Proxy</Text>
                <Text style={styles.version}>Alpha v0.1.0</Text>
            </View>

            {/* Status Card */}
            <View style={styles.statusCard}>
                <Text style={styles.statusIcon}>{statusInfo.icon}</Text>
                <Text style={[styles.statusText, { color: statusInfo.color }]}>
                    {statusInfo.text}
                </Text>
            </View>

            {/* WebView Container */}
            {showWebView && (
                <View style={styles.webviewContainer}>
                    <View style={styles.webviewHeader}>
                        <Text style={styles.webviewTitle}>OBS Login</Text>
                        <TouchableOpacity
                            onPress={() => setShowWebView(false)}
                            style={styles.closeBtn}
                        >
                            <Text style={styles.closeBtnText}>✕</Text>
                        </TouchableOpacity>
                    </View>
                    <HiddenWebView
                        ref={webViewRef}
                        visible={true}
                        onLoginSuccess={handleLoginSuccess}
                        onLog={addLog}
                    />
                </View>
            )}

            {/* Actions */}
            {!showWebView && (
                <View style={styles.actions}>
                    {status === 'not_logged_in' && (
                        <TouchableOpacity style={styles.primaryBtn} onPress={handleLogin}>
                            <Text style={styles.primaryBtnText}>Login to OBS</Text>
                        </TouchableOpacity>
                    )}

                    {status === 'logged_in' && (
                        <>
                            <TouchableOpacity style={styles.successBtn} disabled>
                                <Text style={styles.successBtnText}>Session Active ✓</Text>
                            </TouchableOpacity>
                            <TouchableOpacity style={styles.secondaryBtn} onPress={handleLogout}>
                                <Text style={styles.secondaryBtnText}>Logout</Text>
                            </TouchableOpacity>
                        </>
                    )}

                    {status === 'initializing' && (
                        <ActivityIndicator size="large" color="#6366f1" />
                    )}
                </View>
            )}

            {/* Dev Monitor Toggle */}
            {__DEV__ && !showWebView && (
                <TouchableOpacity
                    style={styles.monitorToggle}
                    onPress={() => setShowMonitor(!showMonitor)}
                >
                    <Text style={styles.monitorToggleText}>
                        {showMonitor ? 'Hide' : 'Show'} Monitor
                    </Text>
                </TouchableOpacity>
            )}

            {/* Visual Monitor Overlay */}
            {showMonitor && (
                <VisualMonitor
                    logs={logs}
                    onClear={() => setLogs([])}
                    onClose={() => setShowMonitor(false)}
                />
            )}
        </SafeAreaView>
    );
};

const styles = StyleSheet.create({
    container: {
        flex: 1,
        backgroundColor: '#0f0f23',
    },
    header: {
        alignItems: 'center',
        paddingTop: 40,
        paddingBottom: 20,
    },
    title: {
        fontSize: 28,
        fontWeight: '700',
        color: '#fff',
        letterSpacing: 1,
    },
    subtitle: {
        fontSize: 14,
        color: '#a1a1aa',
        marginTop: 4,
    },
    version: {
        fontSize: 11,
        color: '#52525b',
        marginTop: 8,
        fontFamily: 'monospace',
    },
    statusCard: {
        backgroundColor: '#1a1a2e',
        marginHorizontal: 20,
        marginTop: 20,
        padding: 24,
        borderRadius: 16,
        alignItems: 'center',
        borderWidth: 1,
        borderColor: '#27273a',
    },
    statusIcon: {
        fontSize: 40,
        marginBottom: 12,
    },
    statusText: {
        fontSize: 18,
        fontWeight: '600',
    },
    webviewContainer: {
        flex: 1,
        margin: 20,
        borderRadius: 12,
        overflow: 'hidden',
        backgroundColor: '#fff',
    },
    webviewHeader: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        backgroundColor: '#1a1a2e',
        padding: 12,
    },
    webviewTitle: {
        color: '#fff',
        fontWeight: '600',
    },
    closeBtn: {
        padding: 4,
    },
    closeBtnText: {
        color: '#a1a1aa',
        fontSize: 18,
    },
    actions: {
        flex: 1,
        justifyContent: 'center',
        paddingHorizontal: 20,
        gap: 12,
    },
    primaryBtn: {
        backgroundColor: '#6366f1',
        paddingVertical: 16,
        borderRadius: 12,
        alignItems: 'center',
    },
    primaryBtnText: {
        color: '#fff',
        fontSize: 16,
        fontWeight: '600',
    },
    successBtn: {
        backgroundColor: 'rgba(34, 197, 94, 0.15)',
        paddingVertical: 16,
        borderRadius: 12,
        alignItems: 'center',
        borderWidth: 1,
        borderColor: '#22c55e',
    },
    successBtnText: {
        color: '#22c55e',
        fontSize: 16,
        fontWeight: '600',
    },
    secondaryBtn: {
        backgroundColor: 'transparent',
        paddingVertical: 12,
        borderRadius: 12,
        alignItems: 'center',
        borderWidth: 1,
        borderColor: '#3f3f46',
    },
    secondaryBtnText: {
        color: '#a1a1aa',
        fontSize: 14,
    },
    monitorToggle: {
        position: 'absolute',
        bottom: 40,
        right: 20,
        backgroundColor: 'rgba(99, 102, 241, 0.2)',
        paddingHorizontal: 12,
        paddingVertical: 6,
        borderRadius: 20,
    },
    monitorToggleText: {
        color: '#6366f1',
        fontSize: 12,
    },
});

export default LauncherScreen;
