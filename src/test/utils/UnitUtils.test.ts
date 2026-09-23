import { describe, expect, it } from "vitest";
import { UnitUtils } from "@/utils/UnitUtils";
import { DistanceUnit, WeightUnit } from "@/utils/backend/models";

describe("UnitUtils weight conversion", () => {
  it("keeps kilograms unchanged", () => {
    expect(UnitUtils.fromKg(80, WeightUnit.Kilograms)).toBe(80);
    expect(UnitUtils.toKg(80, WeightUnit.Kilograms)).toBe(80);
  });

  it("converts kilograms to pounds and back", () => {
    expect(UnitUtils.fromKg(100, WeightUnit.Pounds)).toBeCloseTo(220.462, 3);
    expect(UnitUtils.toKg(220.462, WeightUnit.Pounds)).toBeCloseTo(100, 6);
  });

  it("round-trips arbitrary values", () => {
    const lb = UnitUtils.fromKg(72.5, WeightUnit.Pounds);
    expect(UnitUtils.toKg(lb, WeightUnit.Pounds)).toBeCloseTo(72.5, 10);
  });
});

describe("UnitUtils distance conversion", () => {
  it("keeps kilometers unchanged", () => {
    expect(UnitUtils.fromKm(10, DistanceUnit.Kilometers)).toBe(10);
    expect(UnitUtils.toKm(10, DistanceUnit.Kilometers)).toBe(10);
  });

  it("converts kilometers to miles and back", () => {
    expect(UnitUtils.fromKm(10, DistanceUnit.Miles)).toBeCloseTo(6.21371, 5);
    expect(UnitUtils.toKm(6.21371, DistanceUnit.Miles)).toBeCloseTo(10, 6);
  });
});

describe("UnitUtils.getUnit", () => {
  it.each([
    [DistanceUnit.Kilometers, "Km"],
    [DistanceUnit.Miles, "Mi"],
    [WeightUnit.Kilograms, "Kg"],
    [WeightUnit.Pounds, "Lb"],
  ])("returns the label for %s", (unit, label) => {
    expect(UnitUtils.getUnit(unit)).toBe(label);
  });
});

describe("UnitUtils.SEMICIRCLE_TO_DEGREES", () => {
  it("maps 2^31 semicircles to 180 degrees", () => {
    expect(2 ** 31 * UnitUtils.SEMICIRCLE_TO_DEGREES).toBe(180);
    expect(-(2 ** 30) * UnitUtils.SEMICIRCLE_TO_DEGREES).toBe(-90);
  });
});
