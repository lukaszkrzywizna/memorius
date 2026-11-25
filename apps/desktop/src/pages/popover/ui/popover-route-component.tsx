import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { PopoverPage } from "@/pages/popover/ui/popover-page"
import {
  dayTimelineQuery,
  memoriusQueryKey,
  pauseRecording,
  quitApp,
  recorderStatusQuery,
  resumeRecording,
  showMainWindow,
} from "@/shared/api/memorius-commands"
import { todayIso } from "@/shared/lib/local-date"

export function PopoverRouteComponent() {
  const queryClient = useQueryClient()
  const status = useQuery(recorderStatusQuery())
  const today = useQuery(dayTimelineQuery(todayIso(), 15))
  const refresh = () =>
    queryClient.invalidateQueries({ queryKey: memoriusQueryKey })
  const pause = useMutation({ mutationFn: pauseRecording, onSuccess: refresh })
  const resume = useMutation({
    mutationFn: resumeRecording,
    onSuccess: refresh,
  })
  const open = useMutation({ mutationFn: showMainWindow })
  const quit = useMutation({ mutationFn: quitApp })

  return (
    <PopoverPage
      status={status.data}
      today={today.data}
      onOpen={() => open.mutate()}
      onPause={() => pause.mutate(30)}
      onResume={() => resume.mutate()}
      onQuit={() => quit.mutate()}
    />
  )
}
