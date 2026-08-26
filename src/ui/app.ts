import { createElement, useState } from "react";
import { OnboardingScreen } from "./onboarding/onboarding-screen.ts";
import { SaleScreen } from "./sales/sale-screen.ts";
export const SCREEN = {
  SALES: "sales",
  ONBOARDING: "onboarding",
} as const;

export const NAVIGATION_ACTION = {
  START_ONBOARDING: "start_onboarding",
  RETURN_TO_SALES: "return_to_sales",
} as const;

type Screen = (typeof SCREEN)[keyof typeof SCREEN];
type NavigationAction =
  (typeof NAVIGATION_ACTION)[keyof typeof NAVIGATION_ACTION];

export function screenAfter(
  _current: Screen,
  action: NavigationAction,
): Screen {
  return action === NAVIGATION_ACTION.START_ONBOARDING
    ? SCREEN.ONBOARDING
    : SCREEN.SALES;
}

export function App() {
  const [screen, setScreen] = useState<Screen>(SCREEN.SALES);

  if (screen === SCREEN.ONBOARDING) {
    return createElement(OnboardingScreen, {
      onBack: () =>
        setScreen((current) =>
          screenAfter(current, NAVIGATION_ACTION.RETURN_TO_SALES),
        ),
    });
  }

  return createElement(
    "div",
    null,
    createElement(
      "nav",
      { "aria-label": "Application" },
      createElement(
        "button",
        {
          type: "button",
          onClick: () =>
            setScreen((current) =>
              screenAfter(current, NAVIGATION_ACTION.START_ONBOARDING),
            ),
        },
        "Onboard product",
      ),
    ),
    createElement(SaleScreen),
  );
}
