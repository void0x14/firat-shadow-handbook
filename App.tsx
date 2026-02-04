/**
 * App Entry Point
 * 
 * Fırat Shadow Handbook - Alpha Build
 */

import React from 'react';
import { StyleSheet } from 'react-native';
import { StatusBar } from 'expo-status-bar';
import LauncherScreen from './src/screens/LauncherScreen';

export default function App() {
    return (
        <>
            <StatusBar style="light" />
            <LauncherScreen />
        </>
    );
}

const styles = StyleSheet.create({
    container: {
        flex: 1,
        backgroundColor: '#0f0f23',
    },
});
