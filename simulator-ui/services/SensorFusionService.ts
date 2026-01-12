import { quat, vec3 } from 'gl-matrix';

export class SensorFusionService {
    private currentQ: quat;
    private targetQ: quat;
    private isInitialized: boolean = false;

    constructor() {
        this.currentQ = quat.create();
        this.targetQ = quat.create();
    }

    public getQuaternion(): quat {
        return quat.clone(this.currentQ);
    }

    public update(alpha: number, beta: number, gamma: number): quat {
        // 1. Convert Euler Angles (Degrees) to Quaternion
        // Alpha: Z-axis (Yaw) 0-360
        // Beta: X-axis (Pitch) -180-180
        // Gamma: Y-axis (Roll) -90-90
        
        // Convert to radians
        const radAlpha = alpha * (Math.PI / 180);
        const radBeta = beta * (Math.PI / 180);
        const radGamma = gamma * (Math.PI / 180);

        // Create rotation quaternion from Euler angles
        // Note: DeviceOrientation order is usually Z-X-Y (Alpha-Beta-Gamma)
        const q = quat.create();
        quat.fromEuler(q, beta, alpha, -gamma); // Adjusting mapping for screen space

        if (!this.isInitialized) {
            quat.copy(this.currentQ, q);
            this.isInitialized = true;
        }

        // SLERP for minimal smoothing (faster response)
        quat.slerp(this.currentQ, this.currentQ, q, 0.3);

        // Normalize to prevent drift
        quat.normalize(this.currentQ, this.currentQ);

        return quat.clone(this.currentQ);
    }

    public reset() {
        this.isInitialized = false;
    }
}
