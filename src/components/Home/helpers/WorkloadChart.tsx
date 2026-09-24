import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { WorkoutLoad } from "@/utils/SessionUtils";
import { useContext } from "react";
import {
  Area,
  CartesianGrid,
  ComposedChart,
  Legend,
  Line,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";

type Props = {
  workload: WorkoutLoad[];
  minDate: number;
};

export function WorkloadChart({ workload, minDate }: Props) {
  const { translate } = useContext(I18nSettingsContext);

  return (
    <div className="chart-container ">
      <ResponsiveContainer
        className="chart-responsive"
        width="100%"
        height="100%"
      >
        <ComposedChart data={workload}>
          <CartesianGrid stroke="#80808000" strokeDasharray="5 5" />
          <XAxis
            dataKey="date"
            type="number"
            domain={[minDate, workload[workload.length - 1].date]}
            stroke="#fff"
            tick={false}
            height={0}
          />
          <YAxis
            yAxisId="left"
            stroke="#fff"
            width={0}
            domain={[0, 1]}
            tick={false}
          />{" "}
          <Area
            dataKey="lower"
            stackId="1"
            stroke="none"
            type="monotone"
            legendType="none"
            fill="transparent"
            dot={false}
            isAnimationActive={false}
            activeDot={false}
          />
          <Area
            dataKey="upper"
            stackId="1"
            stroke="none"
            type="monotone"
            fill="lightgreen"
            legendType="none"
            fillOpacity={0.1}
            dot={false}
            isAnimationActive={false}
            activeDot={false}
          />
          <Line
            type="monotone"
            name={translate("workload")}
            dataKey="current"
            stroke="green"
            dot={{ fill: "green" }}
            isAnimationActive={false}
            activeDot={false}
          />
          <Line
            type="monotone"
            name={translate("reference")}
            legendType="line"
            dataKey="reference"
            stroke="#ffffff40"
            dot={false}
            isAnimationActive={false}
            activeDot={false}
          />
          <Legend />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}
