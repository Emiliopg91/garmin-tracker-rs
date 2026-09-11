import { BackendClient } from "@/utils/backend/client";
import { useContext, useEffect, useState } from "react";
import { AppContext } from "@/context/AppContext";
import { BodyMetricListItem } from "@/utils/backend/models";
import { BodyMetricsDetailsModal } from "./BodyMetricsDetailsModal";
import { Button, Checkbox } from "@mui/material";
import { BodyMetricsAddModal } from "./BodyMetricsAddModal";
import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";
import { TimeUtils } from "@/utils/TimeUtils";
import { UnitUtils } from "@/utils/UnitUtils";
import { BodyMetricsCompareModal } from "./BodyMetricsCompareModal";
import "@/styles/BodyMetrics/BodyMetrics.css";

type ChartDataType = {
  date: number;
  fat: number;
  lean: number;
  weight: number;
}[];

export function BodyMetricList() {
  const { startLoading, finishLoading, translate, settings } =
    useContext(AppContext);

  const [bodyMetrics, setBodyMetrics] = useState<BodyMetricListItem[]>([]);
  const [addingNew, setAddingNew] = useState(false);
  const [measureDetails, setMeasureDetails] = useState<
    BodyMetricListItem | undefined
  >(undefined);

  const [chartData, setChartData] = useState<ChartDataType>([]);

  const [minWeight, setMinWeight] = useState(0);
  const [maxWeight, setMaxWeight] = useState(99999);

  const [minLean, setMinLean] = useState(0);
  const [maxLean, setMaxLean] = useState(99999);

  const [minFat, setMinFat] = useState(0);
  const [maxFat, setMaxFat] = useState(100);

  const [minDate, setMinDate] = useState(99999);
  const [maxDate, setMaxDate] = useState(0);

  const [toCompare, setToCompare] = useState<number[]>([]);
  const [comparingSessions, setComparingSessions] = useState<
    BodyMetricListItem[]
  >([]);

  const goToCompare = () => {
    const sorted = toCompare.sort();
    const sessions = [];
    for (let i = 0; i < sorted.length; i++) {
      sessions.push(bodyMetrics.find((entry) => entry.date == sorted[i])!);
    }
    setComparingSessions(sessions);
  };

  const refreshList = () => {
    startLoading();
    BackendClient.getBodyMeasures()
      .then((data) => {
        setBodyMetrics(data);
        const newChartData: ChartDataType = [];
        const data_c = [...data];
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

          if (lMinKg > data.weight) {
            lMinKg = data.weight;
          } else if (lMaxKg < data.weight) {
            lMaxKg = data.weight;
          }

          if (lMinLean > data.lean_mass) {
            lMinLean = data.lean_mass;
          } else if (lMaxLean < data.lean_mass) {
            lMaxLean = data.lean_mass;
          }

          if (lMinFat > data.fat_ratio) {
            lMinFat = data.fat_ratio;
          } else if (lMaxFat < data.fat_ratio) {
            lMaxFat = data.fat_ratio;
          }
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
      })
      .finally(() => {
        finishLoading();
      });
  };

  const toggleSelect = (
    event: React.MouseEvent<HTMLButtonElement>,
    metric: BodyMetricListItem,
  ) => {
    event.stopPropagation();
    if (toCompare.includes(metric.date)) {
      setToCompare([...toCompare].filter((date) => date != metric.date));
    } else {
      setToCompare([...toCompare, metric.date]);
    }
  };

  useEffect(() => {
    refreshList();
  }, []);

  const openModal = (details: BodyMetricListItem) => {
    setMeasureDetails(details);
  };

  return (
    <>
      <div id="list-layer">
        {bodyMetrics.length > 1 && (
          <>
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
          </>
        )}

        <table>
          <thead>
            <tr>
              <th className="text-center">{translate("date")}</th>
              <th className="text-center">{translate("weight")}</th>
              <th className="text-center">{translate("fat_ratio")}</th>
              <th className="text-center">{translate("lean_mass")}</th>
              <th className="text-center">{translate("water_ratio")}</th>
            </tr>
          </thead>
          <tbody>
            {bodyMetrics.map((measure, idx) => (
              <tr
                key={idx}
                className="clickable-row"
                onClick={() => openModal(measure)}
              >
                <td>
                  {bodyMetrics.length > 1 && (
                    <Checkbox
                      onClick={(event) => toggleSelect(event, measure)}
                      className="compare-checkbox"
                      disabled={
                        toCompare.length >= 3 &&
                        !toCompare.includes(measure.date)
                      }
                    />
                  )}
                  {TimeUtils.formatDate(measure.date)}
                </td>
                <td>
                  {UnitUtils.fromKg(
                    measure.weight,
                    settings.weight_unit,
                  ).toFixed(1)}{" "}
                  {UnitUtils.getUnit(settings.weight_unit)}
                </td>
                <td>{measure.fat_ratio.toFixed(1)}%</td>
                <td>
                  {UnitUtils.fromKg(
                    measure.lean_mass,
                    settings.weight_unit,
                  ).toFixed(1)}{" "}
                  {UnitUtils.getUnit(settings.weight_unit)}
                </td>
                <td>{measure.water_ratio.toFixed(1)}%</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div>
        {measureDetails && (
          <BodyMetricsDetailsModal
            measures={measureDetails}
            onClose={() => {
              setMeasureDetails(undefined);
            }}
            onDelete={refreshList}
          />
        )}
        {addingNew && (
          <BodyMetricsAddModal
            onClose={() => {
              setAddingNew(false);
              refreshList();
            }}
            latest={bodyMetrics.length > 0 ? bodyMetrics[0] : undefined}
          />
        )}
        {comparingSessions.length >= 2 && (
          <BodyMetricsCompareModal
            measures={comparingSessions}
            onClose={() => setComparingSessions([])}
          />
        )}
      </div>
      <div className="list-action-bar list-action-bar-row">
        {toCompare.length >= 2 && (
          <Button
            variant="contained"
            className="full-width-button mr-5"
            onClick={goToCompare}
          >
            {translate("compare")}
          </Button>
        )}
        <Button
          id="add-measure-button"
          variant="contained"
          className="full-width-button"
          onClick={() => {
            setAddingNew(true);
          }}
        >
          {translate("add_entry")}
        </Button>
      </div>
    </>
  );
}
