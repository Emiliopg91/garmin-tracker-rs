import { I18nSettingsContext } from "@/context/I18nSettingsContext";
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
  record: boolean;
  row: number;
}

// Chronological row for the Y axis: the month right after the current one
// (the oldest, carried over from last year) sits at row 0 (top); the
// current month sits at row 11 (bottom).
const monthToRow = (month: number, todayMonth: number): number =>
  (month - todayMonth - 1 + 12) % 12;

const rowToMonth = (row: number, todayMonth: number): number =>
  ((row + todayMonth) % 12) + 1;

// The grid only tracks month/day, not year: a month after the current one
// belongs to last year, everything up to and including the current month
// belongs to this year.
const yearForMonth = (
  month: number,
  todayMonth: number,
  todayYear: number,
): number => (month <= todayMonth ? todayYear : todayYear - 1);

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
const HEATMAP_SCALE_MAX = 100;
const HEATMAP_Y_AXIS_WIDTH = 65;
const HEATMAP_X_AXIS_HEIGHT = 10;
const HEATMAP_DAY_TICKS = [1, 10, 20, 30];
const HEATMAP_ROW_TICKS = [0, 2, 4, 6, 8, 10];

const hexToRgb = (hex: string): [number, number, number] => [
  parseInt(hex.slice(1, 3), 16),
  parseInt(hex.slice(3, 5), 16),
  parseInt(hex.slice(5, 7), 16),
];

const getHeatColor = (load: number): string => {
  if (load == 0) {
    const [r1, g1, b1] = hexToRgb(HEATMAP_ZERO_COLOR);
    return `rgba(${r1}, ${g1}, ${b1}, 0.1)`;
  } else {
    const [r1, g1, b1] = hexToRgb(HEATMAP_MAX_COLOR);
    const [r2, g2, b2] = hexToRgb(HEATMAP_MAX_COLOR);
    const alpha = Math.min(1, load / HEATMAP_SCALE_MAX);
    const lerp = (a: number, b: number) => Math.round(a + (b - a) * alpha);
    return `rgba(${lerp(r1, r2)}, ${lerp(g1, g2)}, ${lerp(b1, b2)}, ${alpha})`;
  }
};

interface HeatmapProps {
  data: [number, boolean][][] | undefined;
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
      yearForMonth(point.month, todayMonth, todayYear),
      point.month,
      point.day,
    );

  const points: HeatMapPoint[] = data.flatMap((monthLoads, m) => {
    const month = m + 1;
    return monthLoads.map((data, d) => ({
      month,
      day: d + 1,
      load: data[0],
      record: data[1],
      row: monthToRow(month, todayMonth),
    }));
  });
  const rawCellWidth = (size.width - HEATMAP_Y_AXIS_WIDTH) / 31;
  const rawCellHeight = (size.height - HEATMAP_X_AXIS_HEIGHT) / 12;
  const cellSize = Math.max(0, Math.min(rawCellWidth, rawCellHeight));
  // Shrink the plot area itself to exactly cellSize * 31/12, so consecutive
  // points sit cellSize apart (true square tiling) - the leftover space
  // becomes a margin around the whole grid instead of gaps between cells.
  const extraWidth = Math.max(
    0,
    size.width - HEATMAP_Y_AXIS_WIDTH - cellSize * 31,
  );
  const extraHeight = Math.max(
    0,
    size.height - HEATMAP_X_AXIS_HEIGHT - cellSize * 12,
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
    const isRecord = point.record;
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
            domain={[0.5, 12.5]}
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
              return (
                <div className="heatmap-tooltip">
                  {`${point.day} ${translate("month_" + point.month)}:  ${point.load}`}
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
