import { DistanceUnit, WeightUnit } from "./backend/models";

type UnitType = DistanceUnit | WeightUnit;
export class UnitUtils {
  private static readonly KG_TO_LB = 2.20462;
  private static readonly KM_TO_MI = 0.621371;

  public static fromKg(kg: number, unit: WeightUnit): number {
    switch (unit) {
      case WeightUnit.Pounds:
        return kg * UnitUtils.KG_TO_LB;
      default:
        return kg;
    }
  }

  public static toKg(value: number, unit: WeightUnit): number {
    switch (unit) {
      case WeightUnit.Pounds:
        return value / UnitUtils.KG_TO_LB;
      default:
        return value;
    }
  }

  public static fromKm(km: number, unit: DistanceUnit): number {
    switch (unit) {
      case DistanceUnit.Miles:
        return km * UnitUtils.KM_TO_MI;
      default:
        return km;
    }
  }

  public static toKm(value: number, unit: DistanceUnit): number {
    switch (unit) {
      case DistanceUnit.Miles:
        return value / UnitUtils.KM_TO_MI;
      default:
        return value;
    }
  }

  public static getUnit(unit: UnitType): string {
    switch (unit) {
      case DistanceUnit.Kilometers:
        return "Km";
      case DistanceUnit.Miles:
        return "Mi";
      case WeightUnit.Kilograms:
        return "Kg";
      case WeightUnit.Pounds:
        return "Lb";
      default:
        return "";
    }
  }
}
