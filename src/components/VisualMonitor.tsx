/**
 * VisualMonitor - Developer Debug Overlay
 * 
 * Shows real-time WebView activity logs.
 * Only visible in __DEV__ mode.
 */

import React from 'react';
import { View, Text, FlatList, StyleSheet, TouchableOpacity } from 'react-native';
import type { WebViewLog } from './HiddenWebView';

interface VisualMonitorProps {
    logs: WebViewLog[];
    onClear: () => void;
    onClose: () => void;
}

const LogItem = React.memo(({ log }: { log: WebViewLog }) => {
    const getLogColor = () => {
        switch (log.type) {
            case 'success': return '#22c55e';
            case 'error': return '#ef4444';
            case 'navigation': return '#3b82f6';
            default: return '#a1a1aa';
        }
    };

    const getLogIcon = () => {
        switch (log.type) {
            case 'success': return '✓';
            case 'error': return '✗';
            case 'navigation': return '→';
            default: return '•';
        }
    };

    const time = new Date(log.timestamp).toLocaleTimeString('tr-TR', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
    });

    return (
        <View style={styles.logItem}>
            <Text style={[styles.logIcon, { color: getLogColor() }]}>{getLogIcon()}</Text>
            <Text style={styles.logTime}>[{time}]</Text>
            <Text style={[styles.logMessage, { color: getLogColor() }]} numberOfLines={2}>
                {log.message}
            </Text>
        </View>
    );
});

const VisualMonitor: React.FC<VisualMonitorProps> = ({ logs, onClear, onClose }) => {
    if (!__DEV__) return null;

    return (
        <View style={styles.container}>
            <View style={styles.header}>
                <Text style={styles.title}>🔍 Shadow Monitor</Text>
                <View style={styles.actions}>
                    <TouchableOpacity onPress={onClear} style={styles.actionBtn}>
                        <Text style={styles.actionText}>Clear</Text>
                    </TouchableOpacity>
                    <TouchableOpacity onPress={onClose} style={styles.actionBtn}>
                        <Text style={styles.actionText}>✕</Text>
                    </TouchableOpacity>
                </View>
            </View>

            <FlatList
                data={logs}
                keyExtractor={(_, index) => index.toString()}
                renderItem={({ item }) => <LogItem log={item} />}
                style={styles.list}
                inverted
                showsVerticalScrollIndicator={false}
            />
        </View>
    );
};

const styles = StyleSheet.create({
    container: {
        position: 'absolute',
        top: 50,
        left: 10,
        right: 10,
        maxHeight: 200,
        backgroundColor: 'rgba(0, 0, 0, 0.85)',
        borderRadius: 12,
        overflow: 'hidden',
        borderWidth: 1,
        borderColor: 'rgba(255, 255, 255, 0.1)',
    },
    header: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: 10,
        borderBottomWidth: 1,
        borderBottomColor: 'rgba(255, 255, 255, 0.1)',
    },
    title: {
        color: '#fff',
        fontWeight: '600',
        fontSize: 14,
    },
    actions: {
        flexDirection: 'row',
        gap: 10,
    },
    actionBtn: {
        paddingHorizontal: 8,
        paddingVertical: 4,
    },
    actionText: {
        color: '#a1a1aa',
        fontSize: 12,
    },
    list: {
        maxHeight: 140,
        paddingHorizontal: 10,
    },
    logItem: {
        flexDirection: 'row',
        alignItems: 'flex-start',
        paddingVertical: 4,
        gap: 6,
    },
    logIcon: {
        fontSize: 12,
        width: 14,
    },
    logTime: {
        color: '#71717a',
        fontSize: 10,
        fontFamily: 'monospace',
    },
    logMessage: {
        flex: 1,
        fontSize: 11,
        fontFamily: 'monospace',
    },
});

LogItem.displayName = 'LogItem';

export default VisualMonitor;
