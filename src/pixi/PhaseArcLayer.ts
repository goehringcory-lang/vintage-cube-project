import { Container, Graphics, Text, TextStyle } from "pixi.js";
import type { Theme } from "@/hooks/useTheme";
import { UI_FONT_FAMILY } from "@/themes/typography";
import { hexToNum } from "./colorUtils";
import type { PhaseStripCallbacks, PhaseStripState } from "./PhaseStripLayer";

interface ArcStop {
  label: string;
  angle: number;
  phases: string[];
  shows: string[];
}

const ARC_STOPS: ArcStop[] = [
  { label: "UP", angle: 162, phases: ["upkeep"], shows: ["untap", "upkeep"] },
  { label: "DR", angle: 180, phases: ["draw"], shows: ["draw"] },
  { label: "M1", angle: 198, phases: ["main1"], shows: ["main1"] },
  {
    label: "ATK",
    angle: 330,
    phases: ["combatDeclareAttackers"],
    shows: ["combatBegin", "combatDeclareAttackers"],
  },
  { label: "BLK", angle: 345, phases: ["combatDeclareBlockers"], shows: ["combatDeclareBlockers"] },
  {
    label: "DMG",
    angle: 0,
    phases: ["combatFirstStrikeDamage", "combatDamage"],
    shows: ["combatFirstStrikeDamage", "combatDamage", "combatEnd"],
  },
  { label: "M2", angle: 15, phases: ["main2"], shows: ["main2"] },
  { label: "END", angle: 30, phases: ["endOfTurn"], shows: ["endOfTurn", "cleanup"] },
];

const ARC_RADIUS = 98;
const STOP_RADIUS = 13;
const LABEL_SIZE = 10;

interface StopView {
  root: Container;
  disc: Graphics;
  label: Text;
}

export class PhaseArcLayer {
  readonly container = new Container();
  private theme: Theme;
  private callbacks: PhaseStripCallbacks = {};
  private state: PhaseStripState | null = null;
  private readonly views: StopView[];

  constructor(theme: Theme) {
    this.theme = theme;
    this.container.label = "phaseArc";
    this.container.visible = false;
    this.views = ARC_STOPS.map((stop) => {
      const root = new Container();
      root.eventMode = "static";
      root.cursor = "pointer";
      const disc = new Graphics();
      const label = new Text({ text: stop.label });
      label.anchor.set(0.5);
      root.addChild(disc, label);
      root.on("pointertap", (event) => {
        event.stopPropagation();
        if (!this.state?.isInteractive) return;
        for (const phase of stop.phases) this.callbacks.onToggleSelfPhase?.(phase);
      });
      this.container.addChild(root);
      return { root, disc, label };
    });
  }

  setTheme(theme: Theme): void {
    this.theme = theme;
    this.render();
  }

  setCallbacks(callbacks: PhaseStripCallbacks): void {
    this.callbacks = callbacks;
  }

  update(state: PhaseStripState): void {
    this.state = state;
    this.render();
  }

  place(centerX: number, centerY: number, scale: number): void {
    this.container.position.set(centerX, centerY);
    this.container.scale.set(scale);
  }

  private render(): void {
    const state = this.state;
    const gt = this.theme.gameTheme;
    const accent = hexToNum(state?.isActiveTurn ? gt.activeAction.active : gt.textMuted);
    ARC_STOPS.forEach((stop, i) => {
      const view = this.views[i]!;
      const angle = (stop.angle * Math.PI) / 180;
      view.root.position.set(Math.cos(angle) * ARC_RADIUS, Math.sin(angle) * ARC_RADIUS);
      const current = !!state && stop.shows.includes(state.currentStep);
      const stopped = !!state && stop.phases.some((phase) => state.selfEnabledPhases.has(phase));
      view.disc.clear();
      view.disc.circle(0, 0, STOP_RADIUS);
      view.disc.fill({ color: current ? accent : hexToNum(gt.canvas.shadow), alpha: 0.9 });
      view.disc.circle(0, 0, STOP_RADIUS);
      view.disc.stroke({
        color: stopped ? hexToNum(gt.activeAction.active) : hexToNum(gt.textGhost),
        width: stopped ? 2.5 : 1,
        alpha: stopped ? 1 : 0.6,
      });
      view.label.style = new TextStyle({
        fontFamily: UI_FONT_FAMILY,
        fontSize: LABEL_SIZE,
        fontWeight: "800",
        fill: hexToNum(current ? gt.canvas.shadow : gt.textOnTinted),
      });
    });
  }

  destroy(): void {
    this.container.destroy({ children: true });
  }
}
