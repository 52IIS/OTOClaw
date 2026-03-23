import { readFileSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(__dirname, '..');

const versionPath = resolve(rootDir, 'version.json');
const packagePath = resolve(rootDir, 'package.json');
const tauriConfPath = resolve(rootDir, 'src-tauri', 'tauri.conf.json');

const versionData = JSON.parse(readFileSync(versionPath, 'utf-8'));
const { version, buildNumber, releaseDate } = versionData;

console.log(`📦 同步版本信息: v${version} (build ${buildNumber})`);

const packageData = JSON.parse(readFileSync(packagePath, 'utf-8'));
packageData.version = version;
writeFileSync(packagePath, JSON.stringify(packageData, null, 2) + '\n');
console.log('✅ package.json 已更新');

const tauriConfData = JSON.parse(readFileSync(tauriConfPath, 'utf-8'));
tauriConfData.version = version;
writeFileSync(tauriConfPath, JSON.stringify(tauriConfData, null, 2) + '\n');
console.log('✅ tauri.conf.json 已更新');

console.log(`📅 发布日期: ${releaseDate}`);
console.log('🎉 版本同步完成！');
