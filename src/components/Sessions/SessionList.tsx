import { AppContext } from "@/context/AppContext";
import { LoadingContext } from "@/context/LoadingContext";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { BackendClient } from "@/utils/backend/client";
import { SessionListItem } from "@/utils/backend/models";
import { useContext, useEffect, useState } from "react";
import { SessionModal } from "./SessionModal";
import { SessionRow } from "./helpers/SessionRow";
import { BackendListener } from "@/utils/backend/listener";
import { SessionFrontDetails, SessionUtils } from "@/utils/SessionUtils";
import "@/styles/Sessions/SessionList.css";
import { Button } from "@mui/material";
import { SessionsCompareModal } from "./helpers/SessionsCompareModal";

export function SessionsList() {
  const { sessionsVersion } = useContext(AppContext);
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate, settings } = useContext(I18nSettingsContext);
  const [sessions, setSessions] = useState<SessionListItem[]>([]);
  const [sessionDetails, setSessionDetails] = useState<
    SessionFrontDetails | undefined
  >(undefined);
  const [toCompare, setToCompare] = useState<number[]>([]);
  const [toCompareName, setToCompareName] = useState<string | undefined>(
    undefined,
  );
  const [comparingSessions, setComparingSessions] = useState<
    [SessionFrontDetails, SessionFrontDetails] | undefined
  >(undefined);

  const refreshList = () => {
    startLoading();
    BackendClient.getSessions(null)
      .then((data) => {
        setSessions(data);
      })
      .finally(() => {
        finishLoading();
      });
  };

  useEffect(() => {
    const unregisterSessionLocation = BackendListener.onSessionLocationUpdate(
      (data) => {
        setSessions((prev) => {
          return prev.map((item) => {
            if (item.timestamp !== data.session) return item;
            return { ...item, name: data.location };
          });
        });
      },
    );

    refreshList();

    return () => {
      unregisterSessionLocation();
    };
  }, [sessionsVersion]);

  const getSessionDetails = (timestamp: number) => {
    startLoading();
    BackendClient.getSessionDetails(timestamp)
      .then((details) => {
        setSessionDetails(
          SessionUtils.detailsFromBackend(details, settings.weight_unit),
        );
      })
      .finally(() => {
        finishLoading();
      });
  };

  const toggleSelect = (
    event: React.MouseEvent<HTMLButtonElement>,
    session: SessionListItem,
  ) => {
    event.stopPropagation();
    const was0 = toCompare.length == 0;
    if (toCompare.includes(session.timestamp)) {
      const toComp = [...toCompare].filter((date) => date != session.timestamp);
      if (toComp.length == 0) {
        setToCompareName(undefined);
      }
      setToCompare(toComp);
    } else {
      if (toCompare.length < 2) {
        setToCompare([...toCompare, session.timestamp]);
        if (was0) {
          setToCompareName(session.name);
        }
      }
    }
  };

  const goToCompare = async () => {
    const details = [];
    for (const d of [...toCompare].sort((a, b) => a - b)) {
      const frontDetails = SessionUtils.detailsFromBackend(
        await BackendClient.getSessionDetails(d),
        settings.weight_unit,
      );
      details.push(frontDetails);
    }
    setComparingSessions([details[0], details[1]]);
  };

  return (
    <>
      <div id="list-layer">
        <table>
          <colgroup>
            <col className="col-30" />
            <col />
            <col />
            <col />
            <col />
            <col />
            <col />
          </colgroup>
          <thead>
            <tr>
              <th></th>
              <th></th>
              <th className="text-center">{translate("date")}</th>
              <th className="text-center">{translate("sport")}</th>
              <th className="text-center">{translate("name")}</th>
              <th className="text-center">{translate("active_calories")}</th>
              <th className="text-center">{translate("workout_load")}</th>
            </tr>
          </thead>

          <tbody>
            {sessions.map((session, idx) => (
              <SessionRow
                key={idx}
                session={session}
                onSelect={getSessionDetails}
                compareName={toCompareName}
                onToggleCompare={toggleSelect}
              />
            ))}
          </tbody>
        </table>
      </div>

      <div>
        {sessionDetails && (
          <SessionModal
            session={sessionDetails}
            onClose={() => setSessionDetails(undefined)}
            onUpdate={() => refreshList()}
          />
        )}
      </div>

      {toCompare.length == 2 && (
        <div className="list-action-bar list-action-bar-row">
          <Button
            id="add-measure-button"
            variant="contained"
            className="full-width-button"
            onClick={goToCompare}
          >
            {translate("compare")}
          </Button>
        </div>
      )}
      {comparingSessions && (
        <SessionsCompareModal
          sessions={comparingSessions}
          onClose={() => setComparingSessions(undefined)}
        />
      )}
    </>
  );
}
