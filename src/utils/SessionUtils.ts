import {
  SessionDetails,
  SessionListItem,
  SessionSerie,
  WeightUnit,
} from "./backend/models";
import { TimeUtils } from "./TimeUtils";
import { UnitUtils } from "./UnitUtils";

export interface WorkoutLoad {
  date: number;
  upper: number;
  current: number;
  reference: number;
  lower: number;
}

export interface SessionFrontDetails extends SessionDetails {
  zones_times: number[];
  gps_segments: {
    coordinates: [[number, number], [number, number]];
    color: string;
  }[];
  valid_points: [number, number][];
  start_point: [number, number];
  finish_point: [number, number];
  distance: number;
  speed: number;
  pace: number;
  hrRanges: [number, number, number];
  hrBreathData: {
    idx: number;
    hr: number;
    avg: number;
    color: string;
  }[];
  volume: number;
  exercises: string[];
  grouped_series: Record<string, SessionSerie[]>;
}

export class SessionUtils {
  public static detailsFromBackend(
    backDetails: SessionDetails,
    weightUnit: WeightUnit,
  ): SessionFrontDetails {
    const details: SessionFrontDetails = {
      ...backDetails,
      zones_times: [0, 0, 0, 0, 0],
      gps_segments: [],
      valid_points: [],
      start_point: [0, 0],
      finish_point: [0, 0],
      distance: 0,
      speed: 0,
      pace: 0,
      hrBreathData: [],
      hrRanges: [0, 0, 0],
      volume: 0,
      exercises: [],
      grouped_series: {},
    };

    SessionUtils.handleSeries(details, weightUnit);
    SessionUtils.handleGpsCoordiates(details);
    SessionUtils.handleHeartRate(details);

    return details;
  }

  private static handleSeries(
    details: SessionFrontDetails,
    weightUnit: WeightUnit,
  ) {
    if (details.series) {
      details.series.forEach((_, idx) => {
        const copy = { ...details.series[idx] };
        copy.weight = Number(
          UnitUtils.fromKg(copy.weight, weightUnit).toFixed(1),
        );
        details.series[idx] = copy;
        details.volume += copy.reps * copy.weight;

        const name =
          details.series[idx].ex_cat + "-" + details.series[idx].ex_id;
        if (!details.exercises.includes(name)) {
          details.exercises.push(name);
        }
        if (!details.grouped_series[name]) {
          details.grouped_series[name] = [];
        }
        details.grouped_series[name].push(details.series[idx]);
      });
    }
  }

  private static handleGpsCoordiates(details: SessionFrontDetails) {
    if (details.coordinates) {
      for (let i = 0; i < details.coordinates.length; i++) {
        if (details.coordinates[i]) {
          details.coordinates[i]![0] =
            details.coordinates[i]![0] * UnitUtils.SEMICIRCLE_TO_DEGREES;
          details.coordinates[i]![1] =
            details.coordinates[i]![1] * UnitUtils.SEMICIRCLE_TO_DEGREES;
          details.valid_points.push(details.coordinates[i]!);
        }
      }

      const diffs: number[] = [];
      for (let i = 0; i < details.coordinates.length - 1; i++) {
        if (details.coordinates[i]) {
          const diff = SessionUtils.haversine(
            details.coordinates[i]!,
            details.coordinates[i + 1]!,
          );
          diffs.push(diff);
          details.distance += diff;
        }
      }
      details.speed = details.distance / (details.total_elapsed_time / 3600);
      details.pace = details.total_elapsed_time / 60 / details.distance;

      for (let i = 0; i < details.coordinates.length; i++) {
        if (details.coordinates[i]) {
          details.start_point = details.coordinates[i]!;
          break;
        }
      }

      for (let i = details.coordinates.length - 1; i >= 0; i--) {
        if (details.coordinates[i]) {
          details.finish_point = details.coordinates[i]!;
          break;
        }
      }

      const colors = [];
      for (let i = 0; i < details.coordinates.length - 1; i++) {
        colors.push(240);
      }
      if (details.speeds && details.speeds.length > 0) {
        const speeds: number[] = [];
        for (let i = 0; i < details.speeds.length; i++) {
          if (details.speeds[i]) {
            speeds.push(details.speeds[i]!);
          }
        }
        const minSpeed = Math.min(...speeds);
        for (let i = 0; i < speeds.length; i++) {
          speeds[i] = speeds[i] - minSpeed;
        }
        const maxSpeed = Math.max(...speeds);
        for (let i = 0; i < speeds.length && i < colors.length; i++) {
          colors[i] = 240 - Math.round(240 * (speeds[i] / maxSpeed));
        }
      }

      for (let i = 0; i < details.coordinates.length - 1; i++) {
        if (details.coordinates[i]) {
          details.gps_segments.push({
            coordinates: [details.coordinates[i]!, details.coordinates[i + 1]!],
            color: `hsl(${colors[i]}, 100%, 50%)`,
          });
        }
      }
    }
  }

  private static handleHeartRate(details: SessionFrontDetails) {
    if (details.heart_rates) {
      const validHrs = [];
      for (let i = 0; i < details.heart_rates.length; i++) {
        if (details.heart_rates[i]) {
          validHrs.push(details.heart_rates[i]!);
        } else {
          validHrs.push(0);
        }
      }
      const maxHr = Math.max(189, ...validHrs);

      details.zones_times = [0, 0, 0, 0, 0];
      validHrs.forEach((hr) => {
        const rate = hr / maxHr;
        if (rate < 0.6) {
          details.zones_times[0]++;
        } else if (rate < 0.7) {
          details.zones_times[1]++;
        } else if (rate < 0.8) {
          details.zones_times[2]++;
        } else if (rate < 0.9) {
          details.zones_times[3]++;
        } else {
          details.zones_times[4]++;
        }
      });

      const timeFraction =
        details.total_elapsed_time / details.heart_rates.length;
      let accum = 0;
      details.zones_times.forEach((count, idx) => {
        const time_i = Math.round(count * timeFraction);
        details.zones_times[idx] = time_i;
        accum += time_i;
      });

      details.zones_times[0] += Math.round(details.total_elapsed_time) - accum;

      if (validHrs.length > 0) {
        const nonZeroHrs = validHrs.filter((valor) => valor !== 0);

        details.hrRanges = [
          Math.min(...nonZeroHrs),
          Math.round(
            nonZeroHrs.reduce((acc, valor) => acc + valor, 0) /
              nonZeroHrs.length,
          ),
          Math.max(...nonZeroHrs),
        ];
        const maxHr = Math.max(189, Math.max(...validHrs));

        validHrs.forEach((hr, idx) => {
          let color = "red";

          const rateVal = (hr * 1.0) / (maxHr * 1.0);
          if (rateVal <= 0.6) {
            color = "gray";
          } else if (rateVal <= 0.7) {
            color = "turquoise";
          } else if (rateVal <= 0.8) {
            color = "green";
          } else if (rateVal <= 0.9) {
            color = "orange";
          }

          details.hrBreathData.push({
            idx: idx,
            hr,
            avg: details.hrRanges[1],
            color,
          });
        });
      }
    }
  }

  private static toRadians(grados: number): number {
    return (grados * Math.PI) / 180;
  }

  private static haversine(
    start: [number, number],
    end: [number, number],
  ): number {
    const lat1 = SessionUtils.toRadians(start[0]);
    const lat2 = SessionUtils.toRadians(end[0]);
    const dLat = SessionUtils.toRadians(end[0] - start[0]);
    const dLon = SessionUtils.toRadians(end[1] - start[1]);

    const a =
      Math.sin(dLat / 2) ** 2 +
      Math.cos(lat1) * Math.cos(lat2) * Math.sin(dLon / 2) ** 2;
    const c = 2 * Math.asin(Math.sqrt(a));

    return 6371 * c;
  }

  public static calculateWorkoutLoad(data: SessionListItem[]): WorkoutLoad[] {
    if (data.length === 0) {
      return [];
    } else {
      const startOfDay = (d: Date) =>
        new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();

      const addDays = (ts: number, n: number) => {
        const d = new Date(ts);
        d.setDate(d.getDate() + n);
        return d.getTime();
      };

      const CHRONIC_DAYS = 28;
      const ACUTE_DAYS = 7;
      const ACWR_UPPER_RATIO = 1.4;
      const ACWR_LOWER_RATIO = 0.9;
      const TODAY = startOfDay(new Date());

      const LAMBDA_ACUTE = 2 / (ACUTE_DAYS + 1); // ~0.25
      const LAMBDA_CHRONIC = 2 / (CHRONIC_DAYS + 1); // ~0.069

      let working_data = Array.from(
        data
          .map((s) => {
            const [dd, mm, yyyy] = TimeUtils.formatDate(s.timestamp)
              .split("/")
              .map(Number);
            const date = new Date(yyyy, mm - 1, dd).getTime();

            return { date, load: s.training_load };
          })
          .filter(
            (s) => TODAY - 2 * CHRONIC_DAYS * 24 * 60 * 60 * 1000 <= s.date,
          )
          .reduce((map, s) => {
            map.set(s.date, (map.get(s.date) ?? 0) + s.load);
            return map;
          }, new Map<number, number>()),
        ([date, load]) => ({ date, load }),
      );

      for (
        let dat = addDays(TODAY, -2 * CHRONIC_DAYS + 1);
        dat <= TODAY;
        dat = addDays(dat, 1)
      ) {
        if (!working_data.find(({ date }) => date === dat)) {
          working_data.push({ date: dat, load: 0 });
        }
      }

      working_data = working_data.sort((a, b) => a.date - b.date);

      if (working_data.length === 0) {
        return [];
      } else {
        let ewmaAcute = working_data[0].load;
        let ewmaChronic = working_data[0].load;

        const ewmaSeries: {
          date: number;
          acute: number;
          chronic: number;
        }[] = [
          {
            date: working_data[0].date,
            acute: ewmaAcute,
            chronic: ewmaChronic,
          },
        ];

        for (let i = 1; i < working_data.length; i++) {
          const v = working_data[i].load;
          ewmaAcute = v * LAMBDA_ACUTE + ewmaAcute * (1 - LAMBDA_ACUTE);
          ewmaChronic = v * LAMBDA_CHRONIC + ewmaChronic * (1 - LAMBDA_CHRONIC);

          ewmaSeries.push({
            date: working_data[i].date,
            acute: ewmaAcute,
            chronic: ewmaChronic,
          });
        }

        let load_data = ewmaSeries
          .filter((_, idx) => idx >= CHRONIC_DAYS)
          .map((e) => ({
            date: e.date,
            upper: e.chronic * (ACWR_UPPER_RATIO - ACWR_LOWER_RATIO),
            lower: e.chronic * ACWR_LOWER_RATIO,
            current: e.acute,
            reference: e.chronic,
          }));

        if (load_data.length === 0) {
          return [];
        } else {
          load_data = load_data.map((e) => ({
            ...e,
            current: e.current,
            upper: e.upper,
            lower: e.lower,
            reference: e.reference,
          }));

          return load_data;
        }
      }
    }
  }
}
