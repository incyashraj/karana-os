/**
 * Kāraṇa OS - Action Planner
 * 
 * Converts intents → optimized execution plan with:
 * - Dependency resolution (wallet before transfer, install before open)
 * - Conflict detection (can't record while taking photo)
 * - Resource estimation (battery, network, time)
 * - Risk assessment (will spend money, will take 30 seconds)
 * - Execution order optimization (parallel vs sequential)
 */

import { IntentAction } from './intentClassifier';
import { EnrichedContext } from './contextManager';
import { CompleteSystemState } from './systemState';

// =============================================================================
// Types
// =============================================================================

export interface ActionStep {
  action: IntentAction;
  dependencies: number[];  // Indices of steps that must complete first
  canRunInParallel: boolean;
  estimatedDuration: number;  // milliseconds
  resourceRequirements: ResourceRequirements;
  risks: string[];
}

export interface ResourceRequirements {
  battery: number;  // mAh (rough estimate)
  network: boolean;  // Needs internet?
  camera: boolean;  // Needs camera access?
  storage: number;  // MB needed
  permissions: string[];  // Required permissions
}

export interface ActionPlan {
  steps: ActionStep[];
  totalDuration: number;  // milliseconds (sequential execution)
  parallelDuration: number;  // milliseconds (if parallelized)
  requiresConfirmation: boolean;
  confirmationMessage?: string;
  resourceRequirements: ResourceRequirements;
  risks: string[];
  canExecute: boolean;  // Is plan feasible given current system state?
  blockers: string[];  // What prevents execution
}

export interface Dependency {
  from: number;  // Step index
  to: number;  // Step index it depends on
  reason: string;
}

// =============================================================================
// Action Planner
// =============================================================================

export class ActionPlanner {
  
  /**
   * Main entry point: Convert intents → optimized execution plan
   */
  async plan(
    intents: IntentAction[],
    context: EnrichedContext
  ): Promise<ActionPlan> {
    
    if (intents.length === 0) {
      return this.createEmptyPlan();
    }
    
    // Step 1: Add missing dependencies
    const withDependencies = this.addDependencies(intents, context.systemState);
    
    // Step 2: Create steps with metadata
    const steps = withDependencies.map((intent, idx) => 
      this.createActionStep(intent, idx, withDependencies, context)
    );
    
    // Step 3: Detect dependencies between steps
    const dependencies = this.detectDependencies(steps, context);
    
    // Step 4: Update steps with dependency info
    for (const dep of dependencies) {
      steps[dep.from].dependencies.push(dep.to);
    }
    
    // Step 5: Optimize execution order
    const optimized = this.optimizeOrder(steps);
    
    // Step 6: Calculate durations
    const totalDuration = optimized.reduce((sum, step) => sum + step.estimatedDuration, 0);
    const parallelDuration = this.calculateParallelDuration(optimized);
    
    // Step 7: Aggregate resource requirements
    const resourceRequirements = this.aggregateResources(optimized);
    
    // Step 8: Aggregate risks
    const risks = optimized.flatMap(step => step.risks);
    
    // Step 9: Check if plan is executable
    const validation = this.validatePlan(optimized, context.systemState);
    
    // Step 10: Determine if confirmation needed
    const needsConfirmation = this.needsConfirmation(optimized, risks);
    const confirmationMessage = needsConfirmation 
      ? this.buildConfirmationMessage(optimized, risks)
      : undefined;
    
    return {
      steps: optimized,
      totalDuration,
      parallelDuration,
      requiresConfirmation: needsConfirmation,
      confirmationMessage,
      resourceRequirements,
      risks,
      canExecute: validation.canExecute,
      blockers: validation.blockers,
    };
  }

  /**
   * Add missing dependencies (e.g., create wallet before transfer)
   */
  private addDependencies(
    intents: IntentAction[],
    state: CompleteSystemState
  ): IntentAction[] {
    
    const result: IntentAction[] = [];
    
    for (const intent of intents) {
      // If transferring KARA but no wallet, add wallet creation first
      if (intent.layer === 'BLOCKCHAIN' && intent.operation === 'WALLET_TRANSFER') {
        if (!state.layer3_blockchain.wallet.exists) {
          result.push({
            layer: 'BLOCKCHAIN',
            operation: 'WALLET_CREATE',
            params: {},
            confidence: 1.0,
            reasoning: 'Need wallet before transfer',
          });
        }
      }
      
      // If opening app that's not installed, add installation first
      if (intent.layer === 'APPLICATIONS' && intent.operation === 'ANDROID_OPEN') {
        const appName = intent.params.appName;
        const app = state.layer8_applications.androidApps.find(a => a.name === appName);
        if (app && !app.installed) {
          result.push({
            layer: 'APPLICATIONS',
            operation: 'ANDROID_INSTALL',
            params: { appName },
            confidence: 1.0,
            reasoning: `Must install ${appName} before opening`,
          });
        }
      }
      
      // If analyzing vision but camera inactive, activate camera first
      if (intent.layer === 'INTELLIGENCE' && intent.operation === 'VISION_ANALYZE') {
        if (!state.layer1_hardware.camera.active) {
          result.push({
            layer: 'HARDWARE',
            operation: 'CAMERA_ACTIVATE',
            params: {},
            confidence: 1.0,
            reasoning: 'Need camera for vision analysis',
          });
        }
      }
      
      // Add original intent
      result.push(intent);
    }
    
    return result;
  }

  /**
   * Create action step with metadata
   */
  private createActionStep(
    intent: IntentAction,
    index: number,
    allIntents: IntentAction[],
    context: EnrichedContext
  ): ActionStep {
    
    return {
      action: intent,
      dependencies: [],  // Will be filled later
      canRunInParallel: this.canRunInParallel(intent, allIntents),
      estimatedDuration: this.estimateDuration(intent),
      resourceRequirements: this.estimateResources(intent),
      risks: this.assessRisks(intent, context),
    };
  }

  /**
   * Detect dependencies between steps
   */
  private detectDependencies(
    steps: ActionStep[],
    context: EnrichedContext
  ): Dependency[] {
    
    const dependencies: Dependency[] = [];
    
    for (let i = 0; i < steps.length; i++) {
      for (let j = 0; j < i; j++) {
        const from = steps[i];
        const to = steps[j];
        
        // Camera operations must be sequential
        if (from.action.layer === 'HARDWARE' && from.action.operation.startsWith('CAMERA_') &&
            to.action.layer === 'HARDWARE' && to.action.operation.startsWith('CAMERA_')) {
          dependencies.push({
            from: i,
            to: j,
            reason: 'Camera operations must be sequential',
          });
        }
        
        // Vision analysis depends on camera capture
        if (from.action.operation === 'VISION_ANALYZE' && 
            to.action.operation === 'CAMERA_CAPTURE') {
          dependencies.push({
            from: i,
            to: j,
            reason: 'Vision analysis needs captured image',
          });
        }
        
        // Transfer depends on wallet creation
        if (from.action.operation === 'WALLET_TRANSFER' && 
            to.action.operation === 'WALLET_CREATE') {
          dependencies.push({
            from: i,
            to: j,
            reason: 'Transfer needs wallet',
          });
        }
        
        // Open app depends on installation
        if (from.action.operation === 'ANDROID_OPEN' && 
            to.action.operation === 'ANDROID_INSTALL' &&
            from.action.params.appName === to.action.params.appName) {
          dependencies.push({
            from: i,
            to: j,
            reason: `Open depends on ${to.action.params.appName} installation`,
          });
        }
      }
    }
    
    return dependencies;
  }

  /**
   * Check if action can run in parallel with others
   */
  private canRunInParallel(intent: IntentAction, allIntents: IntentAction[]): boolean {
    // Camera operations must be sequential
    if (intent.layer === 'HARDWARE' && intent.operation.startsWith('CAMERA_')) {
      return false;
    }
    
    // Blockchain operations should be sequential (for clarity)
    if (intent.layer === 'BLOCKCHAIN') {
      return false;
    }
    
    // Most other operations can be parallelized
    return true;
  }

  /**
   * Estimate duration of an operation (milliseconds)
   */
  private estimateDuration(intent: IntentAction): number {
    const estimates: Record<string, number> = {
      // Hardware
      'CAMERA_CAPTURE': 500,
      'CAMERA_RECORD_START': 200,
      'DISPLAY_BRIGHTNESS': 100,
      'POWER_STATUS': 50,
      'AUDIO_VOLUME': 100,
      
      // Network
      'NETWORK_STATUS': 200,
      'BLOCKCHAIN_SYNC': 5000,
      
      // Blockchain
      'WALLET_CREATE': 2000,
      'WALLET_BALANCE': 100,
      'WALLET_TRANSFER': 3000,
      'WALLET_TRANSACTIONS': 500,
      
      // Intelligence
      'VISION_ANALYZE': 1500,
      
      // Interface
      'HUD_SHOW': 100,
      'HUD_HIDE': 100,
      'GESTURE_ENABLE': 300,
      'GAZE_ENABLE': 500,
      'AR_MODE_ENABLE': 1000,
      
      // Applications
      'TIMER_CREATE': 200,
      'TIMER_LIST': 100,
      'NAVIGATION_START': 2000,
      'SETTINGS_OPEN': 300,
      'ANDROID_INSTALL': 10000,
      'ANDROID_OPEN': 1000,
      'ANDROID_CLOSE': 500,
      
      // System Services
      'OTA_CHECK': 2000,
      'OTA_INSTALL': 30000,
      'SECURITY_MODE': 500,
      'DIAGNOSTICS_RUN': 5000,
      
      // Spatial
      'ANCHOR_CREATE': 1000,
      'TAB_OPEN': 800,
    };
    
    return estimates[intent.operation] || 500;
  }

  /**
   * Estimate resource requirements
   */
  private estimateResources(intent: IntentAction): ResourceRequirements {
    const base: ResourceRequirements = {
      battery: 0,
      network: false,
      camera: false,
      storage: 0,
      permissions: [],
    };
    
    // Camera operations
    if (intent.operation.startsWith('CAMERA_')) {
      base.battery = 50;  // mAh
      base.camera = true;
      base.permissions = ['camera'];
      if (intent.operation === 'CAMERA_RECORD_START') {
        base.storage = 100;  // MB for video
      } else {
        base.storage = 5;  // MB for photo
      }
    }
    
    // Vision analysis
    if (intent.operation === 'VISION_ANALYZE') {
      base.battery = 30;
      base.network = true;
      base.permissions = ['camera'];
    }
    
    // Blockchain operations
    if (intent.layer === 'BLOCKCHAIN') {
      base.battery = 20;
      base.network = true;
    }
    
    // App installation
    if (intent.operation === 'ANDROID_INSTALL') {
      base.battery = 100;
      base.network = true;
      base.storage = 200;  // Average app size
      base.permissions = ['storage'];
    }
    
    // Navigation
    if (intent.operation === 'NAVIGATION_START') {
      base.battery = 150;  // GPS drains battery
      base.network = true;
      base.permissions = ['location'];
    }
    
    return base;
  }

  /**
   * Assess risks of an operation
   */
  private assessRisks(intent: IntentAction, context: EnrichedContext): string[] {
    const risks: string[] = [];
    
    // Financial risks
    if (intent.operation === 'WALLET_TRANSFER') {
      const amount = intent.params.amount || 0;
      risks.push(`Will transfer ${amount} KARA`);
      if (amount > context.systemState.layer3_blockchain.wallet.balance * 0.5) {
        risks.push('⚠️ Transferring more than 50% of balance');
      }
    }
    
    // Battery risks
    const resources = this.estimateResources(intent);
    const currentBattery = context.systemState.layer1_hardware.power.batteryLevel * 100;
    if (resources.battery > 50 && currentBattery < 20) {
      risks.push('⚠️ Low battery - operation may drain significantly');
    }
    
    // Storage risks
    const availableStorage = 1000;  // TODO: Get from system state
    if (resources.storage > availableStorage * 0.8) {
      risks.push('⚠️ Low storage space');
    }
    
    // Time risks
    const duration = this.estimateDuration(intent);
    if (duration > 10000) {
      risks.push(`⏱️ Will take ${(duration / 1000).toFixed(0)} seconds`);
    }
    
    // Security risks
    if (intent.operation === 'SECURITY_MODE' && intent.params.mode === 'relaxed') {
      risks.push('🔓 Reducing security level');
    }
    
    // Data risks
    if (intent.operation === 'VISION_ANALYZE') {
      if (!context.userProfile.preferences.visionAnalysisConsent) {
        risks.push('📸 Will send image to AI for analysis');
      }
    }
    
    return risks;
  }

  /**
   * Optimize execution order (topological sort considering dependencies)
   */
  private optimizeOrder(steps: ActionStep[]): ActionStep[] {
    // For now, keep original order (already has dependencies added)
    // TODO: Implement proper topological sort for complex dependencies
    return steps;
  }

  /**
   * Calculate parallel execution duration
   */
  private calculateParallelDuration(steps: ActionStep[]): number {
    // Simple heuristic: longest sequential chain
    let maxChainDuration = 0;
    
    for (let i = 0; i < steps.length; i++) {
      let chainDuration = steps[i].estimatedDuration;
      
      // Add all dependent steps
      for (const depIdx of steps[i].dependencies) {
        chainDuration += steps[depIdx].estimatedDuration;
      }
      
      maxChainDuration = Math.max(maxChainDuration, chainDuration);
    }
    
    return maxChainDuration;
  }

  /**
   * Aggregate resource requirements across all steps
   */
  private aggregateResources(steps: ActionStep[]): ResourceRequirements {
    return steps.reduce((total, step) => ({
      battery: total.battery + step.resourceRequirements.battery,
      network: total.network || step.resourceRequirements.network,
      camera: total.camera || step.resourceRequirements.camera,
      storage: total.storage + step.resourceRequirements.storage,
      permissions: [...new Set([...total.permissions, ...step.resourceRequirements.permissions])],
    }), {
      battery: 0,
      network: false,
      camera: false,
      storage: 0,
      permissions: [],
    });
  }

  /**
   * Validate if plan can be executed given current state
   */
  private validatePlan(
    steps: ActionStep[],
    state: CompleteSystemState
  ): { canExecute: boolean; blockers: string[] } {
    
    const blockers: string[] = [];
    
    // Check battery
    const totalBattery = steps.reduce((sum, s) => sum + s.resourceRequirements.battery, 0);
    const currentBatteryMAh = state.layer1_hardware.power.batteryLevel * 3000;  // Assume 3000mAh total
    if (totalBattery > currentBatteryMAh) {
      blockers.push('Insufficient battery for operation');
    }
    
    // Check network
    const needsNetwork = steps.some(s => s.resourceRequirements.network);
    if (needsNetwork && state.layer2_network.peerCount === 0) {
      blockers.push('No network connection');
    }
    
    // Check camera
    const needsCamera = steps.some(s => s.resourceRequirements.camera);
    if (needsCamera && !state.layer1_hardware.camera.active) {
      // This is OK - we can activate camera
    }
    
    // Check storage (simplified)
    const needsStorage = steps.reduce((sum, s) => sum + s.resourceRequirements.storage, 0);
    // TODO: Get actual available storage from system
    
    return {
      canExecute: blockers.length === 0,
      blockers,
    };
  }

  /**
   * Determine if plan needs user confirmation
   */
  private needsConfirmation(steps: ActionStep[], risks: string[]): boolean {
    // High-stakes operations
    const highStakes = steps.some(s => 
      s.action.layer === 'BLOCKCHAIN' && s.action.operation === 'WALLET_TRANSFER' ||
      s.action.layer === 'SYSTEM_SERVICES' && s.action.operation.includes('SECURITY') ||
      s.action.layer === 'SYSTEM_SERVICES' && s.action.operation === 'OTA_INSTALL' ||
      s.action.operation.includes('DELETE')
    );
    
    // Significant risks
    const hasSignificantRisks = risks.some(r => r.includes('⚠️') || r.includes('🔓'));
    
    // Multi-step operations (>3 steps)
    const isComplex = steps.length > 3;
    
    return highStakes || hasSignificantRisks || isComplex;
  }

  /**
   * Build confirmation message
   */
  private buildConfirmationMessage(steps: ActionStep[], risks: string[]): string {
    const actions = steps.map((s, i) => `${i + 1}. ${s.action.operation}`).join('\n');
    const riskText = risks.length > 0 ? `\n\nRisks:\n${risks.join('\n')}` : '';
    
    return `I'm about to:\n${actions}${riskText}\n\nProceed?`;
  }

  /**
   * Create empty plan (no actions)
   */
  private createEmptyPlan(): ActionPlan {
    return {
      steps: [],
      totalDuration: 0,
      parallelDuration: 0,
      requiresConfirmation: false,
      resourceRequirements: {
        battery: 0,
        network: false,
        camera: false,
        storage: 0,
        permissions: [],
      },
      risks: [],
      canExecute: true,
      blockers: [],
    };
  }
}

// Export singleton
export const actionPlanner = new ActionPlanner();
