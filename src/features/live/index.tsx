import type { Feature } from "../registry";
import { LivePage } from "./LivePage";

export const liveFeature: Feature = {
  id: "live",
  label: "Live",
  path: "/live",
  element: <LivePage />,
};
