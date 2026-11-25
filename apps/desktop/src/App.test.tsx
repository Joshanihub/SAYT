import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { App } from "./App";

describe("desktop shell", () => {
  it("renders the two-deck workspace", () => {
    const markup = renderToStaticMarkup(<App />);

    expect(markup).toContain("DECK A");
    expect(markup).toContain("DECK B");
    expect(markup).toContain("Crossfader");
  });
});
