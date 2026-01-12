import { quat, vec3 } from 'gl-matrix';

export interface WorldAnchor {
    id: string;
    worldPosition: { x: number; y: number; z: number };
    timestamp: number;
    label?: string;
}

export interface CameraPose {
    position: vec3;
    rotation: quat;
    timestamp: number;
}

/**
 * SpatialAnchorService
 * 
 * Manages world-locked anchors for AR applications.
 * Converts between world coordinates and screen coordinates based on camera pose.
 */
export class SpatialAnchorService {
    private anchors: Map<string, WorldAnchor>;
    private cameraPose: CameraPose;
    private originSet: boolean = false;

    // Calibration: Initial camera position is world origin
    private initialRotation: quat;

    constructor() {
        this.anchors = new Map();
        this.cameraPose = {
            position: vec3.fromValues(0, 0, 0),
            rotation: quat.create(),
            timestamp: Date.now()
        };
        this.initialRotation = quat.create();
    }

    /**
     * Set the world origin (call after calibration)
     */
    public setOrigin(rotation: quat) {
        quat.copy(this.initialRotation, rotation);
        this.originSet = true;
    }

    /**
     * Update camera pose from sensors
     */
    public updateCameraPose(rotation: quat, translationDelta: { x: number; y: number; z: number }) {
        if (!this.originSet) return;

        // Relative rotation from calibration
        const relativeRotation = quat.create();
        quat.multiply(relativeRotation, rotation, quat.invert(quat.create(), this.initialRotation));
        quat.normalize(relativeRotation, relativeRotation); // Normalize to prevent accumulation errors
        quat.copy(this.cameraPose.rotation, relativeRotation);

        // Accumulate position from deltas with damping to prevent runaway
        const dampingFactor = 0.98; // Slight damping to prevent drift accumulation
        this.cameraPose.position[0] = this.cameraPose.position[0] * dampingFactor + translationDelta.x;
        this.cameraPose.position[1] = this.cameraPose.position[1] * dampingFactor + translationDelta.y;
        this.cameraPose.position[2] = this.cameraPose.position[2] * dampingFactor + translationDelta.z;

        // Clamp camera position to reasonable bounds (prevent infinite drift)
        const maxDistance = 10.0; // 10 meters from origin
        const distance = Math.sqrt(
            this.cameraPose.position[0] ** 2 + 
            this.cameraPose.position[1] ** 2 + 
            this.cameraPose.position[2] ** 2
        );
        if (distance > maxDistance) {
            const scale = maxDistance / distance;
            this.cameraPose.position[0] *= scale;
            this.cameraPose.position[1] *= scale;
            this.cameraPose.position[2] *= scale;
        }

        this.cameraPose.timestamp = Date.now();
    }

    /**
     * Create or update an anchor at a screen position
     * Converts screen coords -> world coords using current camera pose
     */
    public createAnchor(id: string, screenX: number, screenY: number, depth: number = 2.0, label?: string): WorldAnchor {
        // Convert screen position to world position
        const worldPos = this.screenToWorld(screenX, screenY, depth);

        const anchor: WorldAnchor = {
            id,
            worldPosition: { x: worldPos[0], y: worldPos[1], z: worldPos[2] },
            timestamp: Date.now(),
            label
        };

        this.anchors.set(id, anchor);
        return anchor;
    }

    /**
     * Get screen position for an anchor (or null if behind camera)
     */
    public getAnchorScreenPosition(id: string): { x: number; y: number; depth: number; visible: boolean } | null {
        const anchor = this.anchors.get(id);
        if (!anchor) return null;

        const worldPos = vec3.fromValues(
            anchor.worldPosition.x,
            anchor.worldPosition.y,
            anchor.worldPosition.z
        );

        return this.worldToScreen(worldPos);
    }

    /**
     * Convert screen coordinates to world coordinates
     */
    private screenToWorld(screenX: number, screenY: number, depth: number): vec3 {
        // Normalized Device Coordinates (-1 to 1)
        const ndcX = (screenX / window.innerWidth) * 2 - 1;
        const ndcY = -((screenY / window.innerHeight) * 2 - 1); // Invert Y

        // Simple perspective projection (FOV ~60 degrees)
        const fov = 60 * (Math.PI / 180);
        const aspect = window.innerWidth / window.innerHeight;
        const tanHalfFov = Math.tan(fov / 2);

        // Ray in camera space
        const rayX = ndcX * tanHalfFov * aspect * depth;
        const rayY = ndcY * tanHalfFov * depth;
        const rayZ = -depth; // Forward is negative Z

        const rayCameraSpace = vec3.fromValues(rayX, rayY, rayZ);

        // Rotate ray by camera rotation
        const rayWorldSpace = vec3.create();
        vec3.transformQuat(rayWorldSpace, rayCameraSpace, this.cameraPose.rotation);

        // Add camera position
        const worldPos = vec3.create();
        vec3.add(worldPos, this.cameraPose.position, rayWorldSpace);

        return worldPos;
    }

    /**
     * Convert world coordinates to screen coordinates
     */
    private worldToScreen(worldPos: vec3): { x: number; y: number; depth: number; visible: boolean } {
        // Transform to camera space
        const relativePos = vec3.create();
        vec3.subtract(relativePos, worldPos, this.cameraPose.position);

        // Rotate by inverse camera rotation
        const inverseRotation = quat.create();
        quat.invert(inverseRotation, this.cameraPose.rotation);
        const cameraSpacePos = vec3.create();
        vec3.transformQuat(cameraSpacePos, relativePos, inverseRotation);

        const depth = -cameraSpacePos[2]; // Forward is negative Z

        // Check if behind camera
        if (depth <= 0.1) {
            return { x: 0, y: 0, depth, visible: false };
        }

        // Perspective projection
        const fov = 60 * (Math.PI / 180);
        const aspect = window.innerWidth / window.innerHeight;
        const tanHalfFov = Math.tan(fov / 2);

        const ndcX = cameraSpacePos[0] / (depth * tanHalfFov * aspect);
        const ndcY = -cameraSpacePos[1] / (depth * tanHalfFov); // Invert Y

        // Convert to screen coordinates
        const screenX = ((ndcX + 1) / 2) * window.innerWidth;
        const screenY = ((-ndcY + 1) / 2) * window.innerHeight;

        // Check if on screen
        const visible = screenX >= -200 && screenX <= window.innerWidth + 200 &&
                       screenY >= -200 && screenY <= window.innerHeight + 200;

        return { x: screenX, y: screenY, depth, visible };
    }

    /**
     * Get all anchors with their screen positions
     */
    public getAllAnchorsWithScreenPositions(): Array<{ anchor: WorldAnchor; screen: { x: number; y: number; depth: number; visible: boolean } | null }> {
        const results: Array<{ anchor: WorldAnchor; screen: { x: number; y: number; depth: number; visible: boolean } | null }> = [];
        
        this.anchors.forEach(anchor => {
            const screen = this.getAnchorScreenPosition(anchor.id);
            results.push({ anchor, screen });
        });

        return results;
    }

    /**
     * Remove an anchor
     */
    public removeAnchor(id: string): boolean {
        return this.anchors.delete(id);
    }

    /**
     * Clear all anchors
     */
    public clearAnchors() {
        this.anchors.clear();
    }

    /**
     * Get camera position (for debugging)
     */
    public getCameraPosition(): { x: number; y: number; z: number } {
        return {
            x: this.cameraPose.position[0],
            y: this.cameraPose.position[1],
            z: this.cameraPose.position[2]
        };
    }
}
