/**
 * HiddenWebView - The Shadow Orchestrator
 * 
 * Invisible WebView that handles OBS login and data extraction.
 * Communicates with React Native via postMessage bridge.
 */

import React, { useRef, useCallback, useState, forwardRef, useImperativeHandle } from 'react';
import { View, StyleSheet } from 'react-native';
import { WebView, WebViewNavigation, WebViewMessageEvent } from 'react-native-webview';
import { getUserAgentString } from '../services/UAGenerator';
import { extractOBSCookies } from '../services/CookieManager';

const OBS_LOGIN_URL = 'https://obs.firat.edu.tr/oibs/ogrenci/login.aspx';
const OBS_MAIN_URL = 'https://obs.firat.edu.tr/oibs/ogrenci/ogrencianasayfa.aspx';

export interface HiddenWebViewRef {
    navigateTo: (url: string) => void;
    injectJS: (script: string) => void;
    reload: () => void;
}

export interface WebViewLog {
    timestamp: number;
    type: 'info' | 'success' | 'error' | 'navigation';
    message: string;
}

interface HiddenWebViewProps {
    visible?: boolean; // Set to true for debugging
    onLoginSuccess?: () => void;
    onLoginRequired?: () => void;
    onLog?: (log: WebViewLog) => void;
}

// Injected JavaScript for logging and detection
const BRIDGE_SCRIPT = `
(function() {
  // Override console.log to send to React Native
  const originalLog = console.log;
  console.log = function(...args) {
    originalLog.apply(console, args);
    window.ReactNativeWebView?.postMessage(JSON.stringify({
      type: 'log',
      message: args.join(' ')
    }));
  };
  
  // Detect login form
  const loginForm = document.querySelector('#Form1');
  const isLoginPage = !!document.querySelector('#txtKullaniciAdi');
  
  if (isLoginPage) {
    window.ReactNativeWebView?.postMessage(JSON.stringify({
      type: 'page',
      page: 'login'
    }));
  }
  
  // Detect successful login (main page)
  const isMainPage = !!document.querySelector('.ogrenciinfo');
  if (isMainPage) {
    window.ReactNativeWebView?.postMessage(JSON.stringify({
      type: 'page',
      page: 'main',
      success: true
    }));
  }
  
  // Report page title
  window.ReactNativeWebView?.postMessage(JSON.stringify({
    type: 'title',
    title: document.title
  }));
})();
true;
`;

const HiddenWebView = forwardRef<HiddenWebViewRef, HiddenWebViewProps>((props, ref) => {
    const { visible = false, onLoginSuccess, onLoginRequired, onLog } = props;
    const webViewRef = useRef<WebView>(null);
    const [userAgent, setUserAgent] = useState<string>('');

    // Initialize UA on mount
    React.useEffect(() => {
        getUserAgentString().then(setUserAgent);
    }, []);

    // Expose methods to parent
    useImperativeHandle(ref, () => ({
        navigateTo: (url: string) => {
            webViewRef.current?.injectJavaScript(`window.location.href = '${url}'; true;`);
        },
        injectJS: (script: string) => {
            webViewRef.current?.injectJavaScript(script);
        },
        reload: () => {
            webViewRef.current?.reload();
        },
    }));

    const log = useCallback((type: WebViewLog['type'], message: string) => {
        onLog?.({
            timestamp: Date.now(),
            type,
            message,
        });
    }, [onLog]);

    const handleNavigationChange = useCallback(async (navState: WebViewNavigation) => {
        log('navigation', `Navigated to: ${navState.url}`);

        // Check if we landed on the main page (login successful)
        if (navState.url.includes('ogrencianasayfa')) {
            log('success', 'Login detected! Extracting cookies...');

            try {
                const cookies = await extractOBSCookies();
                log('success', `Session ID: ${cookies.aspNetSessionId?.substring(0, 8)}...`);
                onLoginSuccess?.();
            } catch (error) {
                log('error', `Cookie extraction failed: ${error}`);
            }
        }

        // Check if we're on login page
        if (navState.url.includes('login.aspx')) {
            onLoginRequired?.();
        }
    }, [log, onLoginSuccess, onLoginRequired]);

    const handleMessage = useCallback((event: WebViewMessageEvent) => {
        try {
            const data = JSON.parse(event.nativeEvent.data);

            switch (data.type) {
                case 'log':
                    log('info', `[WebView] ${data.message}`);
                    break;
                case 'page':
                    if (data.page === 'login') {
                        log('info', 'Login page detected');
                    } else if (data.page === 'main' && data.success) {
                        log('success', 'Main page loaded - Login successful!');
                    }
                    break;
                case 'title':
                    log('info', `Page title: ${data.title}`);
                    break;
            }
        } catch {
            log('info', `Raw message: ${event.nativeEvent.data}`);
        }
    }, [log]);

    if (!userAgent) {
        return null; // Wait for UA to be ready
    }

    return (
        <View style={[styles.container, visible ? styles.visible : styles.hidden]}>
            <WebView
                ref={webViewRef}
                source={{ uri: OBS_LOGIN_URL }}
                userAgent={userAgent}
                onNavigationStateChange={handleNavigationChange}
                onMessage={handleMessage}
                injectedJavaScript={BRIDGE_SCRIPT}
                javaScriptEnabled={true}
                domStorageEnabled={true}
                sharedCookiesEnabled={true}
                thirdPartyCookiesEnabled={true}
                incognito={false}
                cacheEnabled={true}
                style={styles.webview}
            />
        </View>
    );
});

const styles = StyleSheet.create({
    container: {
        flex: 1,
    },
    hidden: {
        position: 'absolute',
        width: 1,
        height: 1,
        opacity: 0,
        overflow: 'hidden',
    },
    visible: {
        flex: 1,
    },
    webview: {
        flex: 1,
    },
});

HiddenWebView.displayName = 'HiddenWebView';

export default HiddenWebView;
