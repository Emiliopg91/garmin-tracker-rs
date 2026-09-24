import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { SessionSet } from "@/utils/backend/models";
import { useContext, useEffect, useState } from "react";
import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";

type Props = {
  series: Record<string, SessionSet[]>;
};

export function ExerciseVolumeChart({ series }: Props) {
  const { translate } = useContext(I18nSettingsContext);
  const [chartData, setChartData] = useState<
    { date: number; volume: number; reps: number }[]
  >([]);
  const [maxVol, setMaxVol] = useState(0);
  const [minDate, setMinDate] = useState(99999);
  const [maxDate, setMaxDate] = useState(0);

  useEffect(() => {
    const data: { date: number; volume: number; reps: number }[] = [];
    Object.keys(series).forEach((k) => {
      const date = new Date(parseInt(k.split("\n")[1]));

      let count = 0;
      let weight = 0;
      series[k].forEach((s) => {
        count += s.reps;
        weight += s.reps * s.weight;
      });
      data.push({
        date: date.getTime(),
        volume: weight,
        reps: count,
      });
    });
    data.sort((a, b) => a.date - b.date);
    setChartData(data);
    const dates = [...data].map(({ date }) => {
      return date;
    });
    setMinDate(Math.min(...dates));
    setMaxDate(Math.max(...dates));
    const volumes = data.map(({ volume }) => {
      return volume;
    });
    setMaxVol(Math.max(...volumes));
  }, [series]);

  return (
    <div className="chart-container">
      <ResponsiveContainer width="100%" height="100%">
        <LineChart
          data={chartData}
          margin={{ top: 5, right: 5, left: 5, bottom: 5 }}
        >
          <CartesianGrid stroke="#80808000" strokeDasharray="5 5" />
          <XAxis
            dataKey="date"
            type="number"
            domain={[minDate, maxDate]}
            stroke="#fff"
            tick={false}
            height={0}
          />
          <YAxis
            yAxisId="left"
            stroke="#fff"
            width={0}
            domain={[0, maxVol]}
            tick={false}
          />{" "}
          <YAxis yAxisId="right" stroke="#fff" width={0} tick={false} />{" "}
          <Line
            yAxisId="left"
            type="monotone"
            name={translate("volume")}
            dataKey="volume"
            stroke="#0f0"
            dot={{ fill: "#0f0" }}
            activeDot={{ stroke: "#f0f0f000" }}
            isAnimationActive={false}
          />
          <Line
            yAxisId="right"
            type="monotone"
            name={translate("repetitions")}
            dataKey="reps"
            stroke="#f00"
            dot={{ fill: "#f00" }}
            activeDot={{ stroke: "#f0f0f000" }}
            isAnimationActive={false}
          />
          <Legend />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
