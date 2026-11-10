import { CARD_W } from "@/components/game/game.constants";
import { HAND_CARD_BASE } from "@/components/game/game.styles";
import { OPPONENT_HAND_SCALE } from "../constants";
import type { PlayZoneRect } from "../types";

export const TABLETOP_STAGE = {
  width: 1920,
  height: 1080,
  centerlineFrac: 0.48,
  opponentScale: 0.85,
  battlefieldCardWidth: 165,
  battlefieldCardHeight: 165,
  pileCardWidth: 130,
  minFitScale: 0.7,
  rowGap: 12,
  rowEdgePad: 10,
  minVisibleFrac: 0.4,
  pileColumnWidth: 170,
  stackColumnWidth: 270,
  selfFieldBottom: 960,
  opponentFieldTop: 40,
  handCardWidth: 180,
  handCardHeight: 252,
  handHiddenFrac: 0.45,
  handMaxWidthFrac: 0.46,
  handRotationDeg: 3.5,
  handDropPx: 2,
  handHoverLift: 170,
  handHoverScale: 1.55,
  handNeighborPush: 40,
  opponentHandScale: 0.6,
  avatarCenterY: 850,
  avatarClearance: 140,
  opponentAvatarCenterY: 100,
  backdropDarken: 0.12,
} as const;

export interface TabletopStage {
  x: number;
  y: number;
  scale: number;
  width: number;
  height: number;
  centerlineY: number;
}

export function tabletopStage(viewportWidth: number, viewportHeight: number): TabletopStage {
  const scale = Math.min(
    viewportWidth / TABLETOP_STAGE.width,
    viewportHeight / TABLETOP_STAGE.height,
  );
  const width = TABLETOP_STAGE.width * scale;
  const height = TABLETOP_STAGE.height * scale;
  const x = (viewportWidth - width) / 2;
  const y = (viewportHeight - height) / 2;
  return {
    x,
    y,
    scale,
    width,
    height,
    centerlineY: y + height * TABLETOP_STAGE.centerlineFrac,
  };
}

export function tabletopStageRect(
  stage: TabletopStage,
  x: number,
  y: number,
  width: number,
  height: number,
): PlayZoneRect {
  return {
    x: stage.x + x * stage.scale,
    y: stage.y + y * stage.scale,
    width: width * stage.scale,
    height: height * stage.scale,
  };
}

export function tabletopCardScale(stage: TabletopStage): number {
  return (TABLETOP_STAGE.battlefieldCardWidth / CARD_W) * stage.scale;
}

export function tabletopOpponentHandScale(stage: TabletopStage): number {
  return (
    ((TABLETOP_STAGE.handCardWidth * TABLETOP_STAGE.opponentHandScale) /
      (HAND_CARD_BASE.cardW * OPPONENT_HAND_SCALE)) *
    stage.scale
  );
}
