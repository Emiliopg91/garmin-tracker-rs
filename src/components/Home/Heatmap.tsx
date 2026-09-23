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
}

// GitHub-style ramp: 0 load is a translucent black tint over the box
// background, load >= 100 saturates at GitHub's contribution green
const HEATMAP_MIN_COLOR = "#000000";
const HEATMAP_MIN_ALPHA = 0.25;
const HEATMAP_FUTURE_ALPHA = 0.1;
const HEATMAP_MAX_COLOR = "#00ff00";
const HEATMAP_SCALE_MAX = 100;
const HEATMAP_Y_AXIS_WIDTH = 65;
const HEATMAP_X_AXIS_HEIGHT = 18;
const HEATMAP_DAY_TICKS = [1, 10, 20, 30];
const HEATMAP_MONTH_TICKS = [1, 3, 5, 7, 9, 11];

const hexToRgb = (hex: string): [number, number, number] => [
  parseInt(hex.slice(1, 3), 16),
  parseInt(hex.slice(3, 5), 16),
  parseInt(hex.slice(5, 7), 16),
];

const lerpColor = (
  from: string,
  to: string,
  ratio: number,
  fromAlpha: number,
  toAlpha: number,
): string => {
  const [r1, g1, b1] = hexToRgb(from);
  const [r2, g2, b2] = hexToRgb(to);
  const lerp = (a: number, b: number) => Math.round(a + (b - a) * ratio);
  const alpha = fromAlpha + (toAlpha - fromAlpha) * ratio;
  return `rgba(${lerp(r1, r2)}, ${lerp(g1, g2)}, ${lerp(b1, b2)}, ${alpha.toFixed(2)})`;
};

const getHeatColor = (load: number, isFuture: boolean): string => {
  if (isFuture) {
    return `rgba(0, 0, 0, ${HEATMAP_FUTURE_ALPHA})`;
  }
  return lerpColor(
    HEATMAP_MIN_COLOR,
    HEATMAP_MAX_COLOR,
    Math.min(1, load / HEATMAP_SCALE_MAX),
    HEATMAP_MIN_ALPHA,
    1,
  );
};

interface HeatmapProps {
  data: number[][] | undefined;
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

  const points: HeatMapPoint[] = data.flatMap((row, m) =>
    row.map((load, d) => ({ month: m + 1, day: d + 1, load })),
  );
  const cellWidth = (size.width - HEATMAP_Y_AXIS_WIDTH) / 31;
  const cellHeight = (size.height - HEATMAP_X_AXIS_HEIGHT) / 12;
  const today = new Date();
  const todayMonth = today.getMonth() + 1;
  const todayDay = today.getDate();

  const renderCell = (props: ScatterShapeProps) => {
    const { cx, cy, payload } = props;
    if (cx === undefined || cy === undefined) {
      return null;
    }
    const point = payload as HeatMapPoint;
    const width = Math.max(0, cellWidth - 1);
    const height = Math.max(0, cellHeight - 1);
    const isToday = point.month === todayMonth && point.day === todayDay;
    const isFuture =
      point.month > todayMonth ||
      (point.month === todayMonth && point.day > todayDay);
    return (
      <rect
        x={cx - width / 2}
        y={cy - height / 2}
        width={width}
        height={height}
        rx={1}
        fill={getHeatColor(point.load, isFuture)}
        stroke={isToday ? "#ffffff" : "none"}
        strokeWidth={isToday ? 1.5 : 0}
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
        <ScatterChart margin={{ top: 0, right: 0, bottom: 0, left: 0 }}>
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
            dataKey="month"
            type="number"
            domain={[0.5, 12.5]}
            reversed
            stroke="#fff"
            ticks={HEATMAP_MONTH_TICKS}
            tickFormatter={(value: number) => translate("month_" + value)}
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
                | HeatMapPoint
                | undefined;
              if (!props.active || !point) {
                return null;
              }
              return (
                <div className="heatmap-tooltip">
                  {`${point.month}/${point.day}: ${point.load}`}
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
