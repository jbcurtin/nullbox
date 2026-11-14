import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CopyProvider } from "../../copy/CopyProvider";
import { Beast } from "./Beast";

describe("Beast", () => {
  it("shows the mood in English", () => {
    render(
      <CopyProvider locale="en">
        <Beast mood="kaiju" />
      </CopyProvider>,
    );
    expect(screen.getByText("KAIJU MODE. Evacuate the city.")).toBeInTheDocument();
    expect(document.querySelector("[data-mood=kaiju]")).not.toBeNull();
  });

  it("shows the mood in Japanese", () => {
    render(
      <CopyProvider locale="ja">
        <Beast mood="asleep" />
      </CopyProvider>,
    );
    expect(screen.getByText("おやすみ中。起こさないで。")).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("ja");
  });

  it("shows the mood in tsundere Japanese", () => {
    render(
      <CopyProvider locale="ja-x-tsundere">
        <Beast mood="asleep" />
      </CopyProvider>,
    );
    expect(screen.getByText("寝てるわよ。起こしたら承知しないんだから。")).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("ja-x-tsundere");
  });
});
