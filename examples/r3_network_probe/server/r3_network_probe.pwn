#define FILTERSCRIPT

#include <a_samp>

#define R3_PROBE_OUTBOUND "R3_SDK_OUTBOUND_20260812"
#define R3_PROBE_INCOMING "R3_SDK_INCOMING_20260812"
#define R3_PROBE_DIALOG_REQUEST "R3_SDK_DIALOG_REQUEST_20260812"
#define R3_PROBE_COLOUR 0x6FCF97FF
#define R3_PROBE_DIALOG_ID 25000

public OnFilterScriptInit()
{
    print("[r3_network_probe] ready");
    return 1;
}

public OnPlayerText(playerid, text[])
{
    if (!strcmp(text, R3_PROBE_OUTBOUND, false))
    {
        printf("[r3_network_probe] R3_OUTBOUND_OK playerid=%d", playerid);
        SendClientMessage(playerid, R3_PROBE_COLOUR, R3_PROBE_INCOMING);
        printf("[r3_network_probe] R3_INCOMING_SENT playerid=%d", playerid);
        return 0;
    }
    if (!strcmp(text, R3_PROBE_DIALOG_REQUEST, false))
    {
        ShowPlayerDialog(
            playerid,
            R3_PROBE_DIALOG_ID,
            DIALOG_STYLE_MSGBOX,
            "R3 dialog cache probe",
            "Leave this dialog open while the SDK verifies its active flag.",
            "OK",
            ""
        );
        printf("[r3_network_probe] R3_DIALOG_SENT playerid=%d", playerid);
        return 0;
    }
    return 1;
}
