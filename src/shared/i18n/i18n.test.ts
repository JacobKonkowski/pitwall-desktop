import { describe, expect, it } from "vitest";
import { t } from "./index";

describe("i18n", () => {
  it("returns english strings", () => {
    expect(t("app.name")).toBe("PitWall");
    expect(t("nav.analyze")).toBe("Analyze");
  });
});
