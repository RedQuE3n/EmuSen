namespace EmuSen.Crystal
{
    // The one place a core gives meaning to its own event ids - see EmuSen_Crystal_Scheduler.md §3.
    public interface IScheduleHandler
    {
        // now is the event's scheduled time, which is Scheduler.Now for the duration of the call.
        void OnScheduledEvent(int eventId, long now);
    }
}
