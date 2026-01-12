# Layer 9: System Services

## Overview

The System Services Layer provides essential background services for Kāraṇa OS including OTA updates, security hardening, diagnostics, system recovery, power management, and logging. These services ensure system stability, security, and maintainability.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    LAYER 9: SYSTEM SERVICES                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │   OTA    │  │ Security │  │Diagnostic│  │ Recovery │  │  Power   │ │
│  │ Updates  │  │Hardening │  │ Monitor  │  │  Mode    │  │  Mgmt    │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘ │
│       │             │             │             │             │        │
│       └─────────────┴─────────────┴─────────────┴─────────────┘        │
│                               │                                          │
│                               ▼                                          │
│                   ┌────────────────────────┐                            │
│                   │  System Logger         │                            │
│                   │  Central audit trail   │                            │
│                   └────────────────────────┘                            │
└─────────────────────────────────────────────────────────────────────────┘
```

## Core Services

### 1. OTA Update Service

**Purpose**: Download and install system updates over-the-air.

**Update Process**:
```typescript
interface UpdatePackage {
  version: string;
  size: number;
  checksum: string;
  signature: string;
  changelog: string;
  critical: boolean;
}

class OTAUpdateService {
  private currentVersion = '1.0.0';
  private updateChannel: 'stable' | 'beta' | 'dev' = 'stable';
  
  async checkForUpdates(): Promise<UpdatePackage | null> {
    const response = await fetch(`https://updates.karana.os/${this.updateChannel}/latest.json`);
    const update: UpdatePackage = await response.json();
    
    if (this.isNewerVersion(update.version, this.currentVersion)) {
      return update;
    }
    
    return null;
  }
  
  async downloadUpdate(update: UpdatePackage): Promise<Blob> {
    const response = await fetch(update.url, {
      onProgress: (loaded, total) => {
        this.onDownloadProgress(loaded / total);
      },
    });
    
    const blob = await response.blob();
    
    // Verify checksum
    const checksum = await this.calculateChecksum(blob);
    if (checksum !== update.checksum) {
      throw new Error('Update checksum mismatch');
    }
    
    // Verify signature
    const isValid = await this.verifySignature(blob, update.signature);
    if (!isValid) {
      throw new Error('Update signature invalid');
    }
    
    return blob;
  }
  
  async installUpdate(blob: Blob): Promise<void> {
    // 1. Extract update package
    const files = await this.extractPackage(blob);
    
    // 2. Backup current system
    await this.createBackup();
    
    // 3. Apply update atomically
    try {
      await this.applyUpdate(files);
      
      // 4. Restart system
      this.systemLogger.log('Update installed successfully', 'INFO');
      this.restart();
    } catch (error) {
      // Rollback on failure
      await this.restoreBackup();
      throw error;
    }
  }
  
  private async applyUpdate(files: Map<string, Blob>): Promise<void> {
    for (const [path, content] of files) {
      // Write file atomically
      const tempPath = `${path}.tmp`;
      await this.fileSystem.write(tempPath, content);
      await this.fileSystem.rename(tempPath, path);
    }
  }
  
  private async createBackup(): Promise<void> {
    const systemFiles = await this.fileSystem.listSystemFiles();
    const backup = new Map<string, Blob>();
    
    for (const file of systemFiles) {
      const content = await this.fileSystem.read(file);
      backup.set(file, content);
    }
    
    await this.storageManager.set('system_backup', backup);
  }
}
```

**Differential Updates**:
```typescript
class DifferentialUpdater {
  async createDiff(oldVersion: Blob, newVersion: Blob): Promise<Blob> {
    // Binary diff using bsdiff algorithm
    const oldBytes = new Uint8Array(await oldVersion.arrayBuffer());
    const newBytes = new Uint8Array(await newVersion.arrayBuffer());
    
    return this.bsdiff(oldBytes, newBytes);
  }
  
  async applyDiff(oldVersion: Blob, diff: Blob): Promise<Blob> {
    const oldBytes = new Uint8Array(await oldVersion.arrayBuffer());
    const diffBytes = new Uint8Array(await diff.arrayBuffer());
    
    return new Blob([this.bspatch(oldBytes, diffBytes)]);
  }
}
```

---

### 2. Security Service

**Purpose**: Harden system security and monitor for threats.

**Security Policies**:
```typescript
interface SecurityPolicy {
  enforceStrongAuth: boolean;
  requireBiometric: boolean;
  maxFailedAttempts: number;
  sessionTimeout: number;
  allowRootAccess: boolean;
  enforceEncryption: boolean;
}

class SecurityService {
  private policy: SecurityPolicy = {
    enforceStrongAuth: true,
    requireBiometric: true,
    maxFailedAttempts: 3,
    sessionTimeout: 3600, // 1 hour
    allowRootAccess: false,
    enforceEncryption: true,
  };
  
  async initialize(): Promise<void> {
    // Enable secure boot
    await this.enableSecureBoot();
    
    // Start threat monitoring
    this.startThreatMonitoring();
    
    // Enforce policies
    this.enforcePolicies();
  }
  
  private async enableSecureBoot(): Promise<void> {
    // Verify boot chain integrity
    const bootloader = await this.fileSystem.read('/boot/bootloader');
    const kernel = await this.fileSystem.read('/boot/kernel');
    
    const bootloaderHash = await this.hash(bootloader);
    const kernelHash = await this.hash(kernel);
    
    // Compare with known-good hashes
    if (bootloaderHash !== TRUSTED_BOOTLOADER_HASH) {
      this.systemLogger.log('Bootloader tampered', 'CRITICAL');
      this.enterRecoveryMode();
    }
    
    if (kernelHash !== TRUSTED_KERNEL_HASH) {
      this.systemLogger.log('Kernel tampered', 'CRITICAL');
      this.enterRecoveryMode();
    }
  }
  
  private startThreatMonitoring(): void {
    // Monitor for unusual activity
    setInterval(() => {
      this.checkForThreats();
    }, 10000); // Every 10 seconds
  }
  
  private async checkForThreats(): Promise<void> {
    // Check CPU usage
    const cpuUsage = await this.systemMonitor.getCPUUsage();
    if (cpuUsage > 90) {
      this.systemLogger.log('High CPU usage detected', 'WARNING');
    }
    
    // Check network activity
    const networkActivity = await this.systemMonitor.getNetworkActivity();
    if (networkActivity.outbound > 10 * 1024 * 1024) { // 10 MB/s
      this.systemLogger.log('High network activity detected', 'WARNING');
    }
    
    // Check file integrity
    const criticalFiles = ['/system/kernel', '/system/init', '/system/apps/*'];
    for (const pattern of criticalFiles) {
      const files = await this.fileSystem.glob(pattern);
      for (const file of files) {
        const integrity = await this.checkFileIntegrity(file);
        if (!integrity.valid) {
          this.systemLogger.log(`File ${file} tampered`, 'CRITICAL');
          this.quarantine(file);
        }
      }
    }
  }
}
```

**Encryption**:
```typescript
class EncryptionManager {
  private masterKey: CryptoKey;
  
  async initialize(userPassword: string): Promise<void> {
    // Derive master key from user password
    const salt = await this.getSalt();
    this.masterKey = await crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt,
        iterations: 100000,
        hash: 'SHA-256',
      },
      await crypto.subtle.importKey(
        'raw',
        new TextEncoder().encode(userPassword),
        'PBKDF2',
        false,
        ['deriveKey']
      ),
      { name: 'AES-GCM', length: 256 },
      true,
      ['encrypt', 'decrypt']
    );
  }
  
  async encrypt(data: ArrayBuffer): Promise<EncryptedData> {
    const iv = crypto.getRandomValues(new Uint8Array(12));
    
    const encrypted = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      this.masterKey,
      data
    );
    
    return { ciphertext: encrypted, iv };
  }
  
  async decrypt(encrypted: EncryptedData): Promise<ArrayBuffer> {
    return await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv: encrypted.iv },
      this.masterKey,
      encrypted.ciphertext
    );
  }
}
```

---

### 3. Diagnostics Service

**Purpose**: Monitor system health and collect diagnostics.

**System Monitor**:
```typescript
interface SystemMetrics {
  cpu: {
    usage: number;      // Percentage
    temperature: number; // Celsius
  };
  memory: {
    total: number;      // Bytes
    used: number;
    free: number;
  };
  disk: {
    total: number;
    used: number;
    free: number;
  };
  network: {
    rx: number;         // Bytes received
    tx: number;         // Bytes transmitted
  };
  battery: {
    level: number;      // Percentage
    charging: boolean;
  };
}

class DiagnosticsService {
  async collectMetrics(): Promise<SystemMetrics> {
    return {
      cpu: await this.getCPUMetrics(),
      memory: await this.getMemoryMetrics(),
      disk: await this.getDiskMetrics(),
      network: await this.getNetworkMetrics(),
      battery: await this.getBatteryMetrics(),
    };
  }
  
  private async getCPUMetrics(): Promise<{ usage: number; temperature: number }> {
    // Read from /proc/stat (Linux)
    const stat = await this.fileSystem.read('/proc/stat');
    const usage = this.parseCPUUsage(stat);
    
    // Read temperature sensor
    const temp = await this.hardwareManager.getCPUTemperature();
    
    return { usage, temperature: temp };
  }
  
  private async getMemoryMetrics(): Promise<{ total: number; used: number; free: number }> {
    // Read from /proc/meminfo
    const meminfo = await this.fileSystem.read('/proc/meminfo');
    return this.parseMemInfo(meminfo);
  }
  
  // Generate diagnostic report
  async generateReport(): Promise<DiagnosticReport> {
    const metrics = await this.collectMetrics();
    const logs = await this.systemLogger.getRecentLogs(1000);
    const errors = logs.filter(l => l.level === 'ERROR' || l.level === 'CRITICAL');
    
    return {
      timestamp: Date.now(),
      version: this.systemVersion,
      metrics,
      errors,
      recommendations: this.analyzeMetrics(metrics),
    };
  }
  
  private analyzeMetrics(metrics: SystemMetrics): string[] {
    const recommendations: string[] = [];
    
    if (metrics.cpu.temperature > 80) {
      recommendations.push('CPU temperature is high. Consider reducing workload.');
    }
    
    if (metrics.memory.free < 100 * 1024 * 1024) { // < 100 MB
      recommendations.push('Low memory. Close unused apps.');
    }
    
    if (metrics.disk.free < 500 * 1024 * 1024) { // < 500 MB
      recommendations.push('Low disk space. Delete unnecessary files.');
    }
    
    if (metrics.battery.level < 20 && !metrics.battery.charging) {
      recommendations.push('Battery low. Charge device soon.');
    }
    
    return recommendations;
  }
}
```

**Crash Reporter**:
```typescript
class CrashReporter {
  async reportCrash(error: Error, context: any): Promise<void> {
    const report = {
      timestamp: Date.now(),
      error: {
        message: error.message,
        stack: error.stack,
      },
      context,
      systemMetrics: await this.diagnosticsService.collectMetrics(),
      logs: await this.systemLogger.getRecentLogs(100),
    };
    
    // Save locally
    await this.storageManager.set(`crash_${Date.now()}`, report);
    
    // Send to server (if online)
    try {
      await fetch('https://crashes.karana.os/report', {
        method: 'POST',
        body: JSON.stringify(report),
      });
    } catch (e) {
      // Offline, will retry later
    }
  }
}
```

---

### 4. Recovery Service

**Purpose**: Restore system to working state after failures.

**Recovery Modes**:
```typescript
enum RecoveryMode {
  SAFE_MODE = 'SAFE_MODE',           // Minimal services
  FACTORY_RESET = 'FACTORY_RESET',   // Wipe user data
  RESTORE_BACKUP = 'RESTORE_BACKUP', // Restore from backup
}

class RecoveryService {
  async enterSafeMode(): Promise<void> {
    this.systemLogger.log('Entering safe mode', 'INFO');
    
    // Disable non-essential services
    await this.serviceManager.stopService('social_app');
    await this.serviceManager.stopService('wellness_app');
    
    // Start only critical services
    await this.serviceManager.startService('security_service');
    await this.serviceManager.startService('ota_update_service');
    
    // Show recovery UI
    this.hud.showRecoveryUI();
  }
  
  async factoryReset(): Promise<void> {
    // Confirm with user
    const confirmed = await this.hud.confirm({
      title: 'Factory Reset',
      message: 'This will erase all data. Continue?',
    });
    
    if (!confirmed) return;
    
    this.systemLogger.log('Performing factory reset', 'INFO');
    
    // 1. Backup critical data (if possible)
    try {
      await this.backupCriticalData();
    } catch (e) {
      // Continue even if backup fails
    }
    
    // 2. Wipe user data
    await this.fileSystem.deleteRecursive('/data/user');
    await this.storageManager.clear();
    
    // 3. Reset settings
    await this.settingsManager.resetToDefaults();
    
    // 4. Restart
    this.restart();
  }
  
  async restoreBackup(backupId: string): Promise<void> {
    const backup = await this.storageManager.get(`backup_${backupId}`);
    if (!backup) {
      throw new Error('Backup not found');
    }
    
    // Restore files
    for (const [path, content] of backup.files) {
      await this.fileSystem.write(path, content);
    }
    
    // Restore settings
    await this.settingsManager.restore(backup.settings);
    
    this.systemLogger.log('Backup restored successfully', 'INFO');
    this.restart();
  }
}
```

---

### 5. Power Management Service

**Purpose**: Optimize power consumption and battery life.

**Power Modes**:
```typescript
enum PowerMode {
  PERFORMANCE = 'PERFORMANCE',  // Max performance
  BALANCED = 'BALANCED',        // Balance power/performance
  POWER_SAVER = 'POWER_SAVER',  // Minimize power consumption
}

class PowerManagementService {
  private mode: PowerMode = PowerMode.BALANCED;
  
  setMode(mode: PowerMode): void {
    this.mode = mode;
    
    switch (mode) {
      case PowerMode.PERFORMANCE:
        this.applyPerformanceProfile();
        break;
      case PowerMode.BALANCED:
        this.applyBalancedProfile();
        break;
      case PowerMode.POWER_SAVER:
        this.applyPowerSaverProfile();
        break;
    }
  }
  
  private applyPerformanceProfile(): void {
    this.hardwareManager.setCPUGovernor('performance');
    this.hardwareManager.setDisplayBrightness(1.0);
    this.hardwareManager.setDisplayRefreshRate(90); // 90 FPS
  }
  
  private applyBalancedProfile(): void {
    this.hardwareManager.setCPUGovernor('ondemand');
    this.hardwareManager.setDisplayBrightness(0.7);
    this.hardwareManager.setDisplayRefreshRate(60); // 60 FPS
  }
  
  private applyPowerSaverProfile(): void {
    this.hardwareManager.setCPUGovernor('powersave');
    this.hardwareManager.setDisplayBrightness(0.4);
    this.hardwareManager.setDisplayRefreshRate(30); // 30 FPS
    
    // Disable background services
    this.serviceManager.stopService('social_app');
    this.serviceManager.stopService('wellness_app');
  }
  
  // Automatically adjust based on battery level
  monitorBattery(): void {
    setInterval(async () => {
      const battery = await this.getBatteryLevel();
      
      if (battery < 10) {
        this.setMode(PowerMode.POWER_SAVER);
        this.hud.showAlert({
          type: 'warning',
          title: 'Low Battery',
          message: 'Switched to power saver mode',
        });
      } else if (battery < 30 && this.mode === PowerMode.PERFORMANCE) {
        this.setMode(PowerMode.BALANCED);
      }
    }, 30000); // Check every 30s
  }
}
```

---

### 6. System Logger

**Purpose**: Centralized logging for all system components.

**Logger Implementation**:
```typescript
enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARNING = 2,
  ERROR = 3,
  CRITICAL = 4,
}

interface LogEntry {
  timestamp: number;
  level: LogLevel;
  component: string;
  message: string;
  metadata?: any;
}

class SystemLogger {
  private logs: LogEntry[] = [];
  private maxLogs = 10000;
  private minLevel = LogLevel.INFO;
  
  log(message: string, level: LogLevel | string, component: string, metadata?: any): void {
    const levelEnum = typeof level === 'string' ? LogLevel[level] : level;
    
    if (levelEnum < this.minLevel) return;
    
    const entry: LogEntry = {
      timestamp: Date.now(),
      level: levelEnum,
      component,
      message,
      metadata,
    };
    
    this.logs.push(entry);
    
    // Write to persistent storage (async)
    this.persistLog(entry);
    
    // Trim old logs
    if (this.logs.length > this.maxLogs) {
      this.logs.shift();
    }
    
    // Console output
    this.outputToConsole(entry);
  }
  
  private outputToConsole(entry: LogEntry): void {
    const timestamp = new Date(entry.timestamp).toISOString();
    const levelStr = LogLevel[entry.level];
    
    console.log(`[${timestamp}] [${levelStr}] [${entry.component}] ${entry.message}`);
    
    if (entry.metadata) {
      console.log(entry.metadata);
    }
  }
  
  async getRecentLogs(count: number): Promise<LogEntry[]> {
    return this.logs.slice(-count);
  }
  
  async searchLogs(query: { level?: LogLevel; component?: string; text?: string }): Promise<LogEntry[]> {
    return this.logs.filter(entry => {
      if (query.level !== undefined && entry.level !== query.level) {
        return false;
      }
      if (query.component && entry.component !== query.component) {
        return false;
      }
      if (query.text && !entry.message.includes(query.text)) {
        return false;
      }
      return true;
    });
  }
}
```

---

## Performance Metrics

```
┌─ System Services Performance ───────────┐
│ OTA Update Download: 1-5 MB/s           │
│ Update Install: 10-30s                  │
│ Security Scan: 5-10s                    │
│ Diagnostics Collection: 100-500ms       │
│ Recovery Mode Boot: 5-10s               │
│ Logger Write: <1ms                      │
│                                          │
│ Memory Footprint:                        │
│   OTA Service: 10 MB                     │
│   Security Service: 15 MB                │
│   Diagnostics: 5 MB                      │
│   Logger: 2 MB (+ 10 MB logs)            │
│                                          │
│ Total: ~42 MB                            │
└──────────────────────────────────────────┘
```

---

## Future Development

### Phase 1: Remote Management (Q1 2026)
- Remote diagnostics
- Remote recovery
- Fleet management dashboard

### Phase 2: AI Diagnostics (Q2 2026)
- Predictive maintenance
- Anomaly detection
- Self-healing systems

### Phase 3: Blockchain Auditing (Q3 2026)
- Immutable audit logs
- Tamper-proof crash reports
- Verifiable updates

### Phase 4: Zero-Downtime Updates (Q4 2026)
- A/B partition updates
- Live kernel patching
- Hot-swappable modules

---

## Code References

- `karana-core/src/services/ota.rs`: OTA updates
- `karana-core/src/services/security.rs`: Security hardening
- `karana-core/src/services/diagnostics.rs`: System monitoring

---

## Summary

Layer 9 provides:
- **OTA Updates**: Secure over-the-air system updates
- **Security**: Threat monitoring and encryption
- **Diagnostics**: System health monitoring
- **Recovery**: Safe mode and factory reset
- **Power Management**: Adaptive power profiles
- **Logging**: Centralized audit trail

This layer ensures system reliability, security, and maintainability.