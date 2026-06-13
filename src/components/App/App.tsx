import { AppContext } from "@/context/AppContext";
import { JSX, useContext, useEffect, useState } from "react";
import { NavBar } from "../NavBar/NavBar";
import "@/styles/app.css";
import { RpcUtils } from "@/utils/RpcUtils";
import { Tabs } from "@/models/tabs";
import { WorkoutListItem } from "@/models/workouts";
import { ExerciseListItem } from "@/models/exercises";
import { RecordListItem } from "@/models/records";
import { Button } from "react-bootstrap";

export function App(): JSX.Element {
  const { tab, setTab } = useContext(AppContext);
  const [workouts, setWorkouts] = useState<WorkoutListItem[]>([]);
  const [exercises, setExercises] = useState<ExerciseListItem[]>([]);
  const [records, setRecords] = useState<RecordListItem[]>([]);

  const navBarItems = [
    {
      label: "Workouts",
      onSelected: () => {
        setTab(Tabs.WORKOUTS);
      },
      selected: tab == Tabs.WORKOUTS,
    },
    {
      label: "Exercises",
      onSelected: () => {
        setTab(Tabs.EXERCISES);
      },
      selected: tab == Tabs.EXERCISES,
    },
    {
      label: "Records",
      onSelected: () => {
        setTab(Tabs.RECORDS);
      },
      selected: tab == Tabs.RECORDS,
    },
  ];

  useEffect(() => {
    switch (tab) {
      case Tabs.WORKOUTS:
        RpcUtils.getWorkouts().then((data) => {
          setWorkouts(data);

          RpcUtils.getExercises().then((data) => {
            setExercises(data);

            RpcUtils.getRecords().then((data) => {
              setRecords(data);
            });
          });
        });
    }
  }, []);

  return (
    <>
      <div id="viewport">
        <NavBar items={navBarItems} />
        <div id="list-layer">
          {tab == Tabs.WORKOUTS && (
            <table>
              <thead>
                <th>Exercise</th>
                <th>Date</th>
              </thead>
              <tbody>
                {workouts.map((workout, idx) => (
                  <tr
                    key={idx}
                    onClick={() => {
                      console.log("Aleee");
                    }}
                  >
                    <td>{workout.name}</td>
                    <td>{workout.date}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          {tab == Tabs.EXERCISES && (
            <table>
              <tbody>
                {exercises.map((exercise, idx) => (
                  <tr key={idx}>
                    <td>
                      <a>{exercise.name}</a>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}

          {tab == Tabs.RECORDS && (
            <table>
              <thead>
                <th>Exercise</th>
                <th>PR</th>
                <th>1 RM</th>
              </thead>
              <tbody>
                {tab == Tabs.RECORDS &&
                  records.map((record, idx) => (
                    <tr key={idx} style={{ borderBottom: "1px solid gray" }}>
                      <td style={{ textAlign: "left" }}>{record.exercise}</td>
                      <td style={{ textAlign: "left" }}>
                        {record.reps + "x" + record.weight + " Kg"}
                      </td>
                      <td style={{ textAlign: "left" }}>
                        {Math.floor(record.rm) + " Kg"}
                      </td>
                    </tr>
                  ))}
              </tbody>
            </table>
          )}
        </div>
        <div style={{ padding: "5px" }}>
          <Button id="import-button">Import .fit file</Button>
        </div>
      </div>
    </>
  );
}

export default App;
