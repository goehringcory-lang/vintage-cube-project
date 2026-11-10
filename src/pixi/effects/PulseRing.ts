import { Graphics } from "pixi.js";
import { gsap } from "./gsap";
import { PULSE_RING } from "./config";

export class PulseRing {
  readonly gfx = new Graphics();
  private tween: gsap.core.Tween | null = null;
  private key = "";

  constructor() {
    this.gfx.eventMode = "none";
  }

  show(
    x: number,
    y: number,
    width: number,
    height: number,
    radius: number,
    color: number,
    strokeWidth: number = PULSE_RING.strokeWidth,
  ): void {
    const key = `${x},${y},${width},${height},${radius},${color},${strokeWidth}`;
    if (key === this.key) return;
    const startTween = this.key === "";
    this.key = key;
    this.gfx.clear();
    const unit = strokeWidth / PULSE_RING.strokeWidth;
    for (const layer of PULSE_RING.halo) {
      const offset = layer.offset * unit;
      this.gfx.roundRect(
        x - offset,
        y - offset,
        width + offset * 2,
        height + offset * 2,
        radius + offset,
      );
      this.gfx.stroke({ color, width: layer.width * unit, alpha: layer.alpha });
    }
    this.gfx.roundRect(x, y, width, height, radius);
    this.gfx.stroke({ color, width: strokeWidth });
    if (!startTween) return;
    this.stopTween();
    this.tween = gsap.to(this.gfx, {
      alpha: PULSE_RING.minAlpha,
      duration: PULSE_RING.periodS,
      ease: "sine.inOut",
      yoyo: true,
      repeat: -1,
    });
  }

  hide(): void {
    if (!this.key) return;
    this.key = "";
    this.stopTween();
    this.gfx.clear();
  }

  destroy(): void {
    this.stopTween();
    this.gfx.destroy();
  }

  private stopTween(): void {
    this.tween?.kill();
    this.tween = null;
    this.gfx.alpha = 1;
  }
}
