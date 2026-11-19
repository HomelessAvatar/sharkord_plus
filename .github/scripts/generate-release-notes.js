#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');

function getGitOutput(command) {
  try {
    return execSync(command, { encoding: 'utf8', stdio: ['pipe', 'pipe', 'ignore'] }).trim();
  } catch (e) {
    return '';
  }
}

const currentTag = process.env.GITHUB_REF_NAME || getGitOutput('git describe --tags --exact-match') || 'v0.2.4';
const repo = process.env.GITHUB_REPOSITORY || 'HomelessAvatar/sharkord_plus';

// Find previous tag with matching prefix
let prevTag = '';
try {
  const allTags = getGitOutput('git tag --sort=-creatordate').split('\n').map(t => t.trim()).filter(Boolean);
  const prefixMatch = currentTag.match(/^([a-z]+-v)/i);
  const prefix = prefixMatch ? prefixMatch[1] : (currentTag.startsWith('v') ? 'v' : '');

  const matchingTags = prefix ? allTags.filter(t => t.startsWith(prefix)) : allTags;
  const currentIndex = matchingTags.indexOf(currentTag);
  if (currentIndex >= 0 && currentIndex < matchingTags.length - 1) {
    prevTag = matchingTags[currentIndex + 1];
  } else if (matchingTags.length > 1) {
    prevTag = matchingTags[1];
  }
} catch (e) {
  prevTag = '';
}

const range = prevTag ? `${prevTag}..HEAD` : 'HEAD~15..HEAD';
const logOutput = getGitOutput(`git log ${range} --pretty=format:"%H|%h|%s|%an"`);

const commits = logOutput ? logOutput.split('\n').filter(Boolean).map(line => {
  const [hash, shortHash, subject, author] = line.split('|');
  return { hash, shortHash, subject, author };
}) : [];

function formatCommitMessage(msg, repo) {
  let formatted = msg.trim();
  // Turn issue references like #989 or (fixes #989) into markdown links
  formatted = formatted.replace(/#(\d+)/g, `[#$1](https://github.com/${repo}/issues/$1)`);
  return formatted;
}

const features = [];
const fixes = [];
const infra = [];
const docs = [];
const others = [];

for (const commit of commits) {
  const { hash, shortHash, subject } = commit;
  const commitUrl = `https://github.com/${repo}/commit/${hash}`;
  const commitLink = `[\`${shortHash}\`](${commitUrl})`;
  const formattedMsg = formatCommitMessage(subject, repo);

  const lower = subject.toLowerCase();
  if (lower.startsWith('fix')) {
    fixes.push(`* ${commitLink} **${formattedMsg}**`);
  } else if (lower.startsWith('feat')) {
    features.push(`* ${commitLink} **${formattedMsg}**`);
  } else if (lower.startsWith('docs')) {
    docs.push(`* ${commitLink} ${formattedMsg}`);
  } else if (lower.startsWith('ci') || lower.startsWith('chore') || lower.startsWith('build') || lower.startsWith('style') || lower.startsWith('perf')) {
    infra.push(`* ${commitLink} ${formattedMsg}`);
  } else {
    others.push(`* ${commitLink} ${formattedMsg}`);
  }
}

let titlePrefix = 'VibeShark';
if (currentTag.startsWith('mobile-v')) {
  titlePrefix = 'VibeShark Mobile';
} else if (currentTag.startsWith('desktop-v')) {
  titlePrefix = 'VibeShark Desktop';
} else if (currentTag.startsWith('server-v')) {
  titlePrefix = 'VibeShark Server';
}

let body = `## 🦈 ${titlePrefix} ${currentTag}\n\n`;

if (features.length > 0) {
  body += `### ✨ Features & Enhancements\n`;
  body += features.join('\n') + '\n\n';
}

if (fixes.length > 0) {
  body += `### 🐛 Bug Fixes\n`;
  body += fixes.join('\n') + '\n\n';
}

if (infra.length > 0) {
  body += `### 🚀 Infrastructure, Docker & CI/CD\n`;
  body += infra.join('\n') + '\n\n';
}

if (docs.length > 0) {
  body += `### 📚 Documentation\n`;
  body += docs.join('\n') + '\n\n';
}

if (others.length > 0) {
  body += `### 🔄 Other Changes\n`;
  body += others.join('\n') + '\n\n';
}

if (!currentTag.startsWith('mobile-v')) {
  body += `### 📦 Downloadable Desktop Assets (Native Binaries)\n`;
  body += `* **Windows (x64):** \`sharkord-gpui-windows-x64.exe\` or \`sharkord-gpui-windows-x64.zip\`\n`;
  body += `* **Linux (x64):** \`sharkord-gpui-linux-x64\` or \`sharkord-gpui-linux-x64.tar.gz\`\n`;
  body += `* **macOS (Apple Silicon arm64):** \`sharkord-gpui-macos-arm64.zip\`\n\n`;
}

if (!currentTag.startsWith('desktop-v')) {
  body += `### 📱 Mobile Assets\n`;
  body += `* **Android:** \`sharkord-mobile-android.apk\`\n`;
  body += `* **iOS (Sideload / AltStore / TrollStore / Sideloadly):** \`sharkord-mobile-ios-unsigned.ipa\`\n\n`;
}

if (currentTag.startsWith('server-v') || (!currentTag.startsWith('mobile-v') && !currentTag.startsWith('desktop-v'))) {
  body += `### 🐳 Docker Image (GHCR / SemVer & WUD Ready)\n`;
  const semverTag = currentTag.replace(/^[a-z]+-v/i, '').replace(/^v/, '');
  body += `\`\`\`bash\n# Pull the SemVer release (WUD compatible):\ndocker pull ghcr.io/${repo.toLowerCase()}:${semverTag}\n\`\`\`\n\n`;
}

if (prevTag) {
  body += `**Full Changelog:** https://github.com/${repo}/compare/${prevTag}...${currentTag}\n`;
}

const outputPath = process.argv[2] || 'release-notes.md';
fs.writeFileSync(outputPath, body, 'utf8');
console.log(`Generated release notes at ${outputPath}`);
