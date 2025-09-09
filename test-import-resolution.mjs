#!/usr/bin/env node

// Test script to verify import resolution is working

import path from 'path';
import fs from 'fs';

// Create test files
const testDir = '/tmp/test-imports';
if (!fs.existsSync(testDir)) {
    fs.mkdirSync(testDir, { recursive: true });
}

// Create main.ts with imports
fs.writeFileSync(path.join(testDir, 'main.ts'), `
import { helper } from './utils/helper';
import { Component } from './components/Button';
import axios from 'axios';
import config from './config';

export function main() {
    helper();
}
`);

// Create helper.ts
fs.mkdirSync(path.join(testDir, 'utils'), { recursive: true });
fs.writeFileSync(path.join(testDir, 'utils', 'helper.ts'), `
export function helper() {
    console.log('Helper function');
}
`);

// Create Button component  
fs.mkdirSync(path.join(testDir, 'components'), { recursive: true });
fs.writeFileSync(path.join(testDir, 'components', 'Button.tsx'), `
export const Component = () => {
    return '<button>Click me</button>';
}
`);

// Create config/index.ts
fs.mkdirSync(path.join(testDir, 'config'), { recursive: true });
fs.writeFileSync(path.join(testDir, 'config', 'index.ts'), `
export default {
    apiUrl: 'https://api.example.com'
};
`);

console.log('Test files created at:', testDir);
console.log('\nNow run: pnpm tauri:dev');
console.log(`Then select the folder: ${testDir}`);
console.log('\nExpected results:');
console.log('- Import from ./utils/helper should resolve to utils/helper.ts');
console.log('- Import from ./components/Button should resolve to components/Button.tsx');  
console.log('- Import from ./config should resolve to config/index.ts');
console.log('- Import from axios should be marked as external');