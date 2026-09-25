import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { TimeUtils } from "@/utils/TimeUtils";
import { useCallback, useContext, useEffect, useRef, useState } from "react";
import {
  ResponsiveContainer,
  Scatter,
  ScatterChart,
  Tooltip,
  XAxis,
  YAxis,
  type ScatterShapeProps,
} from "recharts";
import "@/styles/Home/Home.css";

interface HeatMapPoint {
  month: number;
  day: number;
  load: number;
  records: number;
  row: number;
}

// Chronological row for the Y axis. The current month is split in two: its
// days after today haven't happened yet this year, so the cell for that
// (month, day) actually holds last year's data - the oldest in the 12-month
// window - and sits at row 0 (top). Its days up to and including today hold
// this year's data - the newest - and sit at row 12 (bottom). The other 11
// months fill rows 1-11 in between, in chronological order.
const monthToRow = (
  month: number,
  day: number,
  todayMonth: number,
  todayDay: number,
): number => {
  if (month === todayMonth) {
    return day > todayDay ? 0 : 12;
  }
  return ((month - todayMonth - 1 + 12) % 12) + 1;
};

const rowToMonth = (row: number, todayMonth: number): number => {
  if (row === 0 || row === 12) {
    return todayMonth;
  }
  return ((row - 1 + todayMonth) % 12) + 1;
};

// The grid only tracks month/day, not year. For the current month, days
// after today belong to last year (this year's occurrence hasn't happened
// yet); days up to and including today belong to this year. Months before
// the current one belong to this year, months after it belong to last year.
const yearForMonth = (
  month: number,
  day: number,
  todayMonth: number,
  todayDay: number,
  todayYear: number,
): number => {
  if (month === todayMonth) {
    return day > todayDay ? todayYear - 1 : todayYear;
  }
  return month < todayMonth ? todayYear : todayYear - 1;
};

// Catches both the always-invalid days (Feb 30, Apr 31, ...) and Feb 29
// on a non-leap year - the Date constructor already knows the real rule.
const isValidCalendarDate = (
  year: number,
  month: number,
  day: number,
): boolean => {
  const date = new Date(year, month - 1, day);
  return (
    date.getFullYear() === year &&
    date.getMonth() === month - 1 &&
    date.getDate() === day
  );
};

// GitHub-style ramp: 0 load is a translucent black tint over the box
// background, load >= 100 saturates at GitHub's contribution green
const HEATMAP_ZERO_COLOR = "#000000";
const HEATMAP_MAX_COLOR = "#00ff00";
const HEATMAP_SCALE_MAX = 150;
const HEATMAP_Y_AXIS_WIDTH = 65;
const HEATMAP_X_AXIS_HEIGHT = 10;
const HEATMAP_DAY_TICKS = [1, 10, 20, 30];
const HEATMAP_ROW_TICKS = [0, 2, 4, 6, 8, 10, 12];

const hexToRgb = (hex: string): [number, number, number] => [
  parseInt(hex.slice(1, 3), 16),
  parseInt(hex.slice(3, 5), 16),
  parseInt(hex.slice(5, 7), 16),
];

const getHeatColor = (load: number): string => {
  if (load == 0) {
    const [r, g, b] = hexToRgb(HEATMAP_ZERO_COLOR);
    return `rgba(${r}, ${g}, ${b}, 0.1)`;
  } else {
    const [r1, g1, b1] = hexToRgb(HEATMAP_MAX_COLOR);
    const [r2, g2, b2] = hexToRgb(HEATMAP_MAX_COLOR);
    const alpha = Math.min(1, load / HEATMAP_SCALE_MAX);
    const lerp = (a: number, b: number) => Math.round(a + (b - a) * alpha);
    return `rgba(${lerp(r1, r2)}, ${lerp(g1, g2)}, ${lerp(b1, b2)}, ${alpha})`;
  }
};

interface HeatmapProps {
  data: [number, number][][] | undefined;
}

export function Heatmap({ data }: HeatmapProps) {
  const { translate } = useContext(I18nSettingsContext);

  const [size, setSize] = useState({ width: 0, height: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  const measure = useCallback(() => {
    const el = containerRef.current;
    if (!el) {
      return;
    }
    const rect = el.getBoundingClientRect();
    setSize({ width: rect.width, height: rect.height });
  }, []);

  const setContainerRef = useCallback(
    (el: HTMLDivElement | null) => {
      containerRef.current = el;
      if (el) {
        measure();
      }
    },
    [measure],
  );

  useEffect(() => {
    window.addEventListener("resize", measure);
    return () => {
      window.removeEventListener("resize", measure);
    };
  }, [measure]);

  if (!data) {
    return null;
  }

  const today = new Date();
  const todayMonth = today.getMonth() + 1;
  const todayDay = today.getDate();
  const todayYear = today.getFullYear();

  const isValidPoint = (point: HeatMapPoint) =>
    isValidCalendarDate(
      yearForMonth(point.month, point.day, todayMonth, todayDay, todayYear),
      point.month,
      point.day,
    );

  const points: HeatMapPoint[] = data.flatMap((monthLoads, m) => {
    const month = m + 1;
    return monthLoads.map((data, d) => {
      const day = d + 1;
      return {
        month,
        day,
        load: data[0],
        records: data[1],
        row: monthToRow(month, day, todayMonth, todayDay),
      };
    });
  });
  const rawCellWidth = (size.width - HEATMAP_Y_AXIS_WIDTH) / 31;
  const rawCellHeight = (size.height - HEATMAP_X_AXIS_HEIGHT) / 13;
  const cellSize = Math.max(0, Math.min(rawCellWidth, rawCellHeight));
  // Shrink the plot area itself to exactly cellSize * 31/13, so consecutive
  // points sit cellSize apart (true square tiling) - the leftover space
  // becomes a margin around the whole grid instead of gaps between cells.
  const extraWidth = Math.max(
    0,
    size.width - HEATMAP_Y_AXIS_WIDTH - cellSize * 31,
  );
  const extraHeight = Math.max(
    0,
    size.height - HEATMAP_X_AXIS_HEIGHT - cellSize * 13,
  );

  const renderCell = (props: ScatterShapeProps) => {
    const { cx, cy, payload } = props;
    if (cx === undefined || cy === undefined) {
      return null;
    }
    const point = payload as HeatMapPoint;
    if (!isValidPoint(point)) {
      return null;
    }
    const width = Math.max(0, cellSize - 1);
    const height = width;
    const isRecord = point.records > 0;
    const isToday = point.month === todayMonth && point.day === todayDay;
    return (
      <rect
        x={cx - width / 2}
        y={cy - height / 2}
        width={width}
        height={height}
        rx={1}
        fill={getHeatColor(point.load)}
        stroke={isRecord ? "#A0A000" : isToday ? "#ffffff" : "none"}
        strokeWidth={isRecord || isToday ? 2 : 0}
      />
    );
  };

  return (
    <div className="chart-container" ref={setContainerRef}>
      <ResponsiveContainer
        className="chart-responsive"
        width="100%"
        height="100%"
      >
        <ScatterChart
          margin={{
            top: extraHeight / 2,
            bottom: extraHeight / 2,
            left: extraWidth / 2,
            right: extraWidth / 2,
          }}
        >
          <XAxis
            dataKey="day"
            type="number"
            domain={[0.5, 31.5]}
            stroke="#fff"
            ticks={HEATMAP_DAY_TICKS}
            tick={{ fill: "#fff", fontSize: 10 }}
            axisLine={false}
            tickLine={false}
            height={HEATMAP_X_AXIS_HEIGHT}
          />
          <YAxis
            dataKey="row"
            type="number"
            domain={[-0.5, 12.5]}
            reversed
            stroke="#fff"
            ticks={HEATMAP_ROW_TICKS}
            tickFormatter={(value: number) =>
              translate("month_" + rowToMonth(value, todayMonth))
            }
            tick={{ fill: "#fff", fontSize: 10 }}
            axisLine={false}
            tickLine={false}
            width={HEATMAP_Y_AXIS_WIDTH}
          />
          <Tooltip
            cursor={false}
            isAnimationActive={false}
            content={(props) => {
              const point = props.payload?.[0]?.payload as
                HeatMapPoint | undefined;
              if (!props.active || !point || !isValidPoint(point)) {
                return null;
              }
              const year = yearForMonth(
                point.month,
                point.day,
                todayMonth,
                todayDay,
                todayYear,
              );
              const date = new Date(year, point.month - 1, point.day);
              return (
                <div className="heatmap-tooltip">
                  <b>{TimeUtils.formatDate(date.getTime() / 1000)}</b>
                  <br />
                  <span>{translate("workout_load") + ": " + point.load}</span>
                  <br />
                  <span>{translate("records") + ": " + point.records}</span>
                </div>
              );
            }}
          />
          <Scatter data={points} shape={renderCell} isAnimationActive={false} />
        </ScatterChart>
      </ResponsiveContainer>
    </div>
  );
}
