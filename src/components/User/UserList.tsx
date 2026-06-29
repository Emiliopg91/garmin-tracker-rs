import { BackendClient } from "@/utils/backend/client";
import { useContext, useEffect, useState } from "react";
import { AppContext } from "@/context/AppContext";
import { UserListItem } from "@/utils/backend/models";
import { UserDetailsModal } from "./UserDetailsModal";
import { Button } from "react-bootstrap";
import { UserAddModal } from "./UserAddModal";

export function UserList() {
  const { setLoading } = useContext(AppContext);

  const [userMeasures, setUserMeasures] = useState<UserListItem[]>([]);
  const [addingNew, setAddingNew] = useState(false);
  const [measureDetails, setMeasureDetails] = useState<
    UserListItem | undefined
  >(undefined);

  const refreshList = () => {
    BackendClient.getUserMeasures()
      .then((data) => {
        setUserMeasures(data);
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
              <td>{measure.weight} Kg</td>
              <td>{measure.fat_ratio}%</td>
              <td>{measure.lean_mass} Kg</td>
              <td>{measure.water_ratio}%</td>
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
