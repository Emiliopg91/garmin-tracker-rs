import { BackendClient } from "@/utils/backend/client";
import { useContext, useEffect, useState } from "react";
import { AppContext } from "@/context/AppContext";
import { UserListItem } from "@/utils/backend/models";
import { UserDetailsModal } from "./UserDetailsModal";
import { Button } from "react-bootstrap";
import { UserAddModal } from "./UserAddModal";
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
  idx: number;
  fat: number;
  lean: number;
  weight: number;
}[];

export function UserList() {
  const { setLoading } = useContext(AppContext);

  const [userMeasures, setUserMeasures] = useState<UserListItem[]>([]);
  const [addingNew, setAddingNew] = useState(false);
  const [measureDetails, setMeasureDetails] = useState<
    UserListItem | undefined
  >(undefined);

  const [chartData, setChartData] = useState<ChartDataType>([]);

  const [minWeight, setMinWeight] = useState(0);
  const [maxWeight, setMaxWeight] = useState(99999);

  const [minLean, setMinLean] = useState(0);
  const [maxLean, setMaxLean] = useState(99999);

  const [minFat, setMinFat] = useState(0);
  const [maxFat, setMaxFat] = useState(100);

  const refreshList = () => {
    BackendClient.getUserMeasures()
      .then((data) => {
        setUserMeasures(data);
        const newChartData: ChartDataType = [];
        const data_c = [...data];
        data_c.reverse();
        data_c.forEach((data, idx) => {
          newChartData.push({
            idx: idx,
            fat: data.fat_ratio,
            lean: data.lean_mass,
            weight: data.weight,
          });

          let lMinKg = 99999;
          let lMaxKg = 0;
          if (lMinKg > data.weight) {
            lMinKg = data.weight;
          } else if (lMaxKg < data.weight) {
            lMaxKg = data.weight;
          }
          setMaxWeight(lMaxKg);
          setMinWeight(lMinKg);

          lMinKg = 99999;
          lMaxKg = 0;
          if (lMinKg > data.lean_mass) {
            lMinKg = data.lean_mass;
          } else if (lMaxKg < data.lean_mass) {
            lMaxKg = data.lean_mass;
          }
          setMaxLean(lMaxKg);
          setMinLean(lMinKg);

          let lMinFat = 99999;
          let lMaxFat = 0;
          if (lMinFat > data.fat_ratio) {
            lMinFat = data.fat_ratio;
          } else if (lMaxFat < data.fat_ratio) {
            lMaxFat = data.fat_ratio;
          }
          setMaxFat(lMaxFat);
          setMinFat(lMinFat);
        });
        setChartData(newChartData);
      })
      .finally(() => {
        setLoading(false);
      });
  };

  useEffect(() => {
    refreshList();
  }, []);

  const openModal = (details: UserListItem) => {
    setMeasureDetails(details);
  };

  return (
    <>
      {userMeasures.length > 0 && (
        <>
          <div style={{ width: "100%", height: 200 }}>
            <ResponsiveContainer width="100%" height="100%">
              <LineChart
                data={chartData}
                margin={{ top: 5, right: 5, left: 5, bottom: 5 }}
              >
                <CartesianGrid stroke="#80808000" strokeDasharray="5 5" />
                <XAxis dataKey="idx" stroke="#fff" tick={false} height={0} />
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
                  name="Fat %"
                  type="monotone"
                  dataKey="fat"
                  stroke="#f00"
                  dot={{ fill: "#f00" }}
                  activeDot={{ stroke: "#00ff0000" }}
                  isAnimationActive={false}
                />
                <Line
                  yAxisId="weight"
                  name="Body weight"
                  type="monotone"
                  dataKey="weight"
                  stroke="cyan"
                  dot={{ fill: "cyan" }}
                  activeDot={{ stroke: "#00ff0000" }}
                  isAnimationActive={false}
                />
                <Line
                  yAxisId="lean"
                  name="Lean mass"
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
            <th style={{ textAlign: "center" }}>Date</th>
            <th style={{ textAlign: "center" }}>Weight</th>
            <th style={{ textAlign: "center" }}>Fat ratio</th>
            <th style={{ textAlign: "center" }}>Lean mass</th>
            <th style={{ textAlign: "center" }}>Water ratio</th>
          </tr>
        </thead>
        <tbody>
          {userMeasures.map((measure, idx) => (
            <tr
              key={idx}
              style={{ cursor: "pointer" }}
              onClick={() => openModal(measure)}
            >
              <td>{measure.date}</td>
              <td>{measure.weight.toFixed(1)} Kg</td>
              <td>{measure.fat_ratio.toFixed(1)}%</td>
              <td>{measure.lean_mass.toFixed(1)} Kg</td>
              <td>{measure.water_ratio.toFixed(1)}%</td>
            </tr>
          ))}
        </tbody>
      </table>

      <div>
        {measureDetails && (
          <UserDetailsModal
            measures={measureDetails}
            onClose={() => setMeasureDetails(undefined)}
          />
        )}
        {addingNew && (
          <UserAddModal
            onClose={() => {
              setAddingNew(false);
              refreshList();
            }}
            latest={userMeasures.length > 0 ? userMeasures[0] : undefined}
          />
        )}
      </div>
      <div style={{ padding: "5px", width: "100%", marginTop: "auto" }}>
        <Button
          id="add-measure-button"
          style={{ width: "100%" }}
          onClick={() => {
            setAddingNew(true);
          }}
        >
          Add entry
        </Button>
      </div>
    </>
  );
}
