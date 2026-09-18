import { AppContext } from "@/context/AppContext";
import { LoadingContext } from "@/context/LoadingContext";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { BackendClient } from "@/utils/backend/client";
import { SessionListItem } from "@/utils/backend/models";
import { useContext, useEffect, useState } from "react";
import { SessionModal } from "./SessionModal";
import EmojiEventsIcon from "@mui/icons-material/EmojiEvents";
import { BackendListener } from "@/utils/backend/listener";
import { TimeUtils } from "@/utils/TimeUtils";
import { SessionFrontDetails, SessionUtils } from "@/utils/SessionUtils";

export function SessionsList() {
  const { sessionsVersion } = useContext(AppContext);
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate, settings } = useContext(I18nSettingsContext);
  const [sessions, setSessions] = useState<SessionListItem[]>([]);
  const [sessionDetails, setSessionDetails] = useState<
    SessionFrontDetails | undefined
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

  return (
    <>
      <div id="list-layer">
        <table>
          <thead>
            <tr>
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
              <tr
                key={idx}
                onClick={() => getSessionDetails(session.timestamp)}
                className="clickable-row"
              >
                <td>
                  {session.has_record && (
                    <EmojiEventsIcon className="trophy-icon" />
                  )}
                </td>
                <td>{TimeUtils.formatTimeDate(session.timestamp)}</td>
                <td>
                  {translate("sport_" + session.sport) +
                    " - " +
                    translate(
                      "sport_" + session.sport + "_" + session.sub_sport,
                    )}
                </td>
                <td>{session.name}</td>
                <td>{session.active_calories}</td>
                <td>{session.training_load}</td>
              </tr>
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
    </>
  );
}
