import jsfeat from 'jsfeat';

export class VisionService {
    private width: number;
    private height: number;
    private prevPyramid: any;
    private currPyramid: any;
    private prevKeypoints: any;
    private currKeypoints: any;
    private pointCount: number;
    private pointStatus: Uint8Array;
    private isInitialized: boolean = false;
    private ctx: CanvasRenderingContext2D | null = null;
    private lastFlowX: number = 0;
    private lastFlowY: number = 0;
    private frameCount: number = 0;
    private frameSkip: number = 0; // Skip frames for performance
    private skipInterval: number = 2; // Process every 2nd frame

    constructor(width: number = 320, height: number = 240) {
        this.width = width;
        this.height = height;
        
        // Initialize JSFeat data structures
        this.prevPyramid = new jsfeat.pyramid_t(3);
        this.currPyramid = new jsfeat.pyramid_t(3);
        this.prevPyramid.allocate(width, height, jsfeat.U8_t | jsfeat.C1_t);
        this.currPyramid.allocate(width, height, jsfeat.U8_t | jsfeat.C1_t);

        this.pointCount = 0;
        this.prevKeypoints = new Float32Array(100 * 2); // Track up to 100 points
        this.currKeypoints = new Float32Array(100 * 2);
        this.pointStatus = new Uint8Array(100);
    }

    public processFrame(video: HTMLVideoElement): { x: number, y: number, z: number, points?: Array<{x: number, y: number}> } {
        // Skip frames for performance
        this.frameSkip++;
        if (this.frameSkip < this.skipInterval) {
            return { x: 0, y: 0, z: 0 };
        }
        this.frameSkip = 0;

        if (!this.ctx) {
            const canvas = document.createElement('canvas');
            canvas.width = this.width;
            canvas.height = this.height;
            this.ctx = canvas.getContext('2d', { willReadFrequently: true });
            if (!this.ctx) return { x: 0, y: 0, z: 0 };
        }

        // Draw video to small canvas
        this.ctx.drawImage(video, 0, 0, this.width, this.height);
        const imageData = this.ctx.getImageData(0, 0, this.width, this.height);

        // Swap pyramids
        const tempPyramid = this.prevPyramid;
        this.prevPyramid = this.currPyramid;
        this.currPyramid = tempPyramid;

        // Swap keypoints
        const tempKeypoints = this.prevKeypoints;
        this.prevKeypoints = this.currKeypoints;
        this.currKeypoints = tempKeypoints;

        // Convert to Grayscale (Pyramid Level 0)
        jsfeat.imgproc.grayscale(imageData.data, this.width, this.height, this.currPyramid.data[0]);

        // Build Pyramid
        this.currPyramid.build(this.currPyramid.data[0], true);

        // If not initialized or lost too many points, find new features
        if (!this.isInitialized || this.pointCount < 15) { // Even lower threshold
            this.findFeatures();
            this.isInitialized = true;
            return { x: 0, y: 0, z: 0 };
        }

        // Optical Flow (Lucas-Kanade) - optimized params
        const winSize = 10; // Smaller for speed
        const maxIter = 20;  // Fewer iterations
        const epsilon = 0.03; // Looser convergence
        const minEigen = 0.001;

        jsfeat.optical_flow_lk.track(
            this.prevPyramid, this.currPyramid,
            this.prevKeypoints, this.currKeypoints,
            this.pointCount, winSize, maxIter,
            this.pointStatus, epsilon, minEigen
        );

        // Calculate Average Motion
        let dx = 0;
        let dy = 0;
        let validPoints = 0;
        const activePoints: Array<{x: number, y: number}> = [];

        // Filter and accumulate
        let newPointCount = 0;
        for (let i = 0; i < this.pointCount; i++) {
            if (this.pointStatus[i] === 1) {
                const px = this.prevKeypoints[i * 2];
                const py = this.prevKeypoints[i * 2 + 1];
                const cx = this.currKeypoints[i * 2];
                const cy = this.currKeypoints[i * 2 + 1];

                // Calculate velocity
                const vx = cx - px;
                const vy = cy - py;

                // Reject outliers (too fast movement usually means error or motion blur)
                if (Math.abs(vx) < 50 && Math.abs(vy) < 50) {
                    dx += vx;
                    dy += vy;
                    validPoints++;
                    
                    // Keep valid points for next frame
                    this.currKeypoints[newPointCount * 2] = cx;
                    this.currKeypoints[newPointCount * 2 + 1] = cy;
                    activePoints.push({ x: cx, y: cy });
                    newPointCount++;
                }
            }
        }

        this.pointCount = newPointCount;

        if (validPoints > 0) {
            const avgDx = dx / validPoints;
            const avgDy = dy / validPoints;

            // Simple exponential smoothing (faster than median)
            const alpha = 0.5; // Increased for responsiveness
            this.lastFlowX = alpha * avgDx + (1 - alpha) * this.lastFlowX;
            this.lastFlowY = alpha * avgDy + (1 - alpha) * this.lastFlowY;

            // Convert pixels to meters
            const fovDegrees = 60;
            const avgDepth = 2.0;
            const pixelsPerDegree = this.width / fovDegrees;
            const metersPerDegree = Math.tan((Math.PI / 180)) * avgDepth;
            const metersPerPixel = metersPerDegree / pixelsPerDegree;

            // Simple threshold
            const threshold = 0.5; // Lower threshold for sensitivity
            if (Math.abs(this.lastFlowX) < threshold) this.lastFlowX = 0;
            if (Math.abs(this.lastFlowY) < threshold) this.lastFlowY = 0;

            // INVERTED: Feature motion is opposite to Camera motion
            // If features move LEFT (-x), Camera moved RIGHT (+x)
            return {
                x: -this.lastFlowX * metersPerPixel,
                y: -this.lastFlowY * metersPerPixel,
                z: 0,
                points: activePoints
            };
        }

        // Faster decay
        this.lastFlowX *= 0.5;
        this.lastFlowY *= 0.5;

        return { x: 0, y: 0, z: 0, points: activePoints };
    }

    private findFeatures() {
        // Faster feature detection
        jsfeat.yape06.laplacian_threshold = 25;
        jsfeat.yape06.min_eigen_value_threshold = 25;

        this.pointCount = jsfeat.yape06.detect(
            this.currPyramid.data[0], 
            this.currKeypoints, 
            this.width, this.height, 
            25, 25 // Larger border = fewer features = faster
        );
        
        // Cap at 50 points for speed
        if (this.pointCount > 50) this.pointCount = 50;
    }
}
