import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { BodyMetricListItem } from "@/utils/backend/models";
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

type ChartDataType = {
  date: number;
  fat: number;
  lean: number;
  weight: number;
}[];

type Props = {
  metrics: BodyMetricListItem[];
};

export function BodyMetricsChart({ metrics }: Props) {
  const { translate } = useContext(I18nSettingsContext);

  const [chartData, setChartData] = useState<ChartDataType>([]);

  const [minWeight, setMinWeight] = useState(0);
  const [maxWeight, setMaxWeight] = useState(99999);

  const [minLean, setMinLean] = useState(0);
  const [maxLean, setMaxLean] = useState(99999);

  const [minFat, setMinFat] = useState(0);
  const [maxFat, setMaxFat] = useState(100);

  const [minDate, setMinDate] = useState(99999);
  const [maxDate, setMaxDate] = useState(0);

  useEffect(() => {
    const newChartData: ChartDataType = [];
    const data_c = [...metrics];
    data_c.reverse();
    let lMinKg = 99999;
    let lMaxKg = 0;
    let lMinFat = 99999;
    let lMaxFat = 0;
    let lMinLean = 99999;
    let lMaxLean = 0;
    data_c.forEach((data) => {
      const date = new Date(data.date * 1000);
      newChartData.push({
        date: date.getTime(),
        fat: data.fat_ratio,
        lean: data.lean_mass,
        weight: data.weight,
      });

      lMinKg = Math.min(lMinKg, data.weight);
      lMaxKg = Math.max(lMaxKg, data.weight);

      lMinLean = Math.min(lMinLean, data.lean_mass);
      lMaxLean = Math.max(lMaxLean, data.lean_mass);

      lMinFat = Math.min(lMinFat, data.fat_ratio);
      lMaxFat = Math.max(lMaxFat, data.fat_ratio);
    });
    const dates = [...newChartData].map(({ date }) => {
      return date;
    });
    setMinDate(Math.min(...dates));
    setMaxDate(Math.max(...dates));
    setMaxWeight(lMaxKg);
    setMinWeight(lMinKg);
    setMaxLean(lMaxLean);
    setMinLean(lMinLean);
    setMaxFat(lMaxFat);
    setMinFat(lMinFat);
    setChartData(newChartData);
  }, [metrics]);

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
            yAxisId="fat"
            stroke="#fff"
            width={0}
            domain={[minFat, maxFat]}
            tick={false}
          />
          <YAxis
            yAxisId="weight"
            stroke="#fff"
            width={0}
            domain={[minWeight, maxWeight]}
            tick={false}
          />
          <YAxis
            yAxisId="lean"
            stroke="#fff"
            width={0}
            domain={[minLean, maxLean]}
            tick={false}
          />
          <Line
            yAxisId="fat"
            name={translate("fat_ratio")}
            type="monotone"
            dataKey="fat"
            stroke="#f00"
            dot={{ fill: "#f00" }}
            activeDot={{ stroke: "#00ff0000" }}
            isAnimationActive={false}
          />
          <Line
            yAxisId="weight"
            name={translate("body_weight")}
            type="monotone"
            dataKey="weight"
            stroke="cyan"
            dot={{ fill: "cyan" }}
            activeDot={{ stroke: "#00ff0000" }}
            isAnimationActive={false}
          />
          <Line
            yAxisId="lean"
            name={translate("lean_mass")}
            type="monotone"
            dataKey="lean"
            stroke="green"
            dot={{ fill: "green" }}
            activeDot={{ stroke: "#00ff0000" }}
            isAnimationActive={false}
          />
          <Legend />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
