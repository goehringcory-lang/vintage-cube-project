import { BATTLEFIELD_MIN_ROWS_LARGEST, FIELD_INNER_EDGE_PAD_PX } from "../constants";
import { battlefieldScaleForMultiplier, scaleForRowsWithCombatRow } from "../GridLayout";
import { computeBoardLayout } from "./boardLayout";
import type {
  BattlefieldLayoutInput,
  BattlefieldLayoutPolicy,
  BattlefieldLayoutResult,
} from "./battlefieldLayoutPolicy";
import {
  TABLETOP_STAGE,
  tabletopCardScale,
  tabletopStage,
  tabletopStageRect,
} from "./tabletopStage";
import { GAME_CARD_SIZES } from "@/components/game/game.constants";

function computeTabletop(input: BattlefieldLayoutInput): BattlefieldLayoutResult {
  const stage = tabletopStage(input.width, input.height);
  const fieldX = TABLETOP_STAGE.pileColumnWidth;
  const fieldWidth = TABLETOP_STAGE.width - fieldX - TABLETOP_STAGE.stackColumnWidth;
  const centerline = TABLETOP_STAGE.height * TABLETOP_STAGE.centerlineFrac;
  const self = tabletopStageRect(
    stage,
    fieldX,
    centerline,
    fieldWidth,
    TABLETOP_STAGE.selfFieldBottom - centerline,
  );
  const opponent = tabletopStageRect(
    stage,
    fieldX,
    TABLETOP_STAGE.opponentFieldTop,
    fieldWidth,
    centerline - TABLETOP_STAGE.opponentFieldTop,
  );
  const selfScale = tabletopCardScale(stage);
  return {
    layout: {
      self,
      opponents: [{ rect: opponent, orientation: "top" }],
      dividerY: stage.centerlineY,
      stripBandPx: 0,
      opponentLayout: "focused",
      tabletop: true,
    },
    scales: { self: selfScale, opponent: selfScale * TABLETOP_STAGE.opponentScale },
    combatRowReserved: false,
    handScale: (TABLETOP_STAGE.handCardWidth / GAME_CARD_SIZES.hand.width) * stage.scale,
    selfClusterMaxHeight: Math.max(input.observedHandReserve, input.requestedBottomReserve),
  };
}

export const DESKTOP_BATTLEFIELD_LAYOUT: BattlefieldLayoutPolicy = {
  showPhaseDivider: true,

  effectiveBottomReserve(requestedBottomReserve) {
    return requestedBottomReserve;
  },
  clusterHeight(_self, observedHandReserve, requestedBottomReserve) {
    return Math.max(observedHandReserve, requestedBottomReserve);
  },

  compute(input): BattlefieldLayoutResult {
    if (input.opponentCount === 1) return computeTabletop(input);
    const layout = computeBoardLayout(
      input.width,
      input.height,
      input.opponentCount,
      input.requestedBottomReserve,
      false,
      0.5,
      input.opponentLayout,
    );
    const playmatTrim = (usable: number) => Math.max(1, usable - FIELD_INNER_EDGE_PAD_PX);
    const selfUsable = playmatTrim(Math.max(1, layout.self.height - input.requestedBottomReserve));
    const selfScale = Math.max(
      Number.EPSILON,
      Math.min(
        battlefieldScaleForMultiplier(selfUsable, input.cardSizeMultiplier),
        scaleForRowsWithCombatRow(selfUsable, BATTLEFIELD_MIN_ROWS_LARGEST),
      ),
    );
    const opponentUsables = layout.opponents.map((opponent) =>
      playmatTrim(Math.max(1, opponent.rect.height)),
    );
    const opponentUsable = opponentUsables.length ? Math.min(...opponentUsables) : selfUsable;
    const opponentScale = Math.max(
      Number.EPSILON,
      layout.opponentLayout === "overview"
        ? scaleForRowsWithCombatRow(opponentUsable, 1)
        : Math.min(
            battlefieldScaleForMultiplier(opponentUsable, input.cardSizeMultiplier),
            scaleForRowsWithCombatRow(opponentUsable, BATTLEFIELD_MIN_ROWS_LARGEST),
          ),
    );
    return {
      layout,
      scales: { self: selfScale, opponent: opponentScale },
      combatRowReserved: true,
      handScale: input.handViewportScale,
      selfClusterMaxHeight: Math.max(input.observedHandReserve, input.requestedBottomReserve),
    };
  },
};
