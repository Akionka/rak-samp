#define FILTERSCRIPT

#include <a_samp>

#define R5_PROBE_OUTBOUND "R5_SDK_OUTBOUND_20260812"
#define R5_PROBE_INCOMING "R5_SDK_INCOMING_20260812"
#define R5_PROBE_COLOUR 0x6FCF97FF

public OnFilterScriptInit()
{
    print("[r5_network_probe] ready");
    return 1;
}

public OnPlayerText(playerid, text[])
{
    if (!strcmp(text, R5_PROBE_OUTBOUND, false))
    {
        printf("[r5_network_probe] R5_OUTBOUND_OK playerid=%d", playerid);
        SendClientMessage(playerid, R5_PROBE_COLOUR, R5_PROBE_INCOMING);
        printf("[r5_network_probe] R5_INCOMING_SENT playerid=%d", playerid);
        return 0;
    }
    return 1;
}
