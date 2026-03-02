import type { AppStatus } from "../types";

interface Props {
  status: AppStatus;
}

export default function MeetingIndicator({ status }: Props) {
  return (
    <div className="flex items-center gap-1 shrink-0">
      <div
        className={`w-1.5 h-1.5 rounded-full ${
          status.isInMeeting ? "bg-red-500 animate-pulse" : "bg-gray-600"
        }`}
      />
      <span
        className={`text-[10px] ${
          status.isInMeeting ? "text-red-400" : "text-gray-500"
        }`}
      >
        {status.isInMeeting ? "Meeting" : "No Meeting"}
      </span>
    </div>
  );
}
