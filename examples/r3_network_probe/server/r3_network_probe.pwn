#define FILTERSCRIPT

#include <a_samp>

#define R3_PROBE_OUTBOUND "R3_SDK_OUTBOUND_20260812"
#define R3_PROBE_INCOMING "R3_SDK_INCOMING_20260812"
#define R3_PROBE_DIALOG_REQUEST "R3_SDK_DIALOG_REQUEST_20260812"
#define R3_PROBE_ENTITY_REQUEST "R3_SDK_ENTITY_REQUEST_20260813"
#define R3_PROBE_COLOUR 0x6FCF97FF
#define R3_PROBE_DIALOG_ID 25000

public OnFilterScriptInit()
{
    print("[r3_network_probe] ready");
    return 1;
}

public OnPlayerCommandText(playerid, cmdtext[])
{
    if (strfind(cmdtext, "/goto ", true) == 0)
    {
        new targetid = strval(cmdtext[6]);
        if (!IsPlayerConnected(targetid) || targetid == playerid)
        {
            SendClientMessage(playerid, R3_PROBE_COLOUR, "Usage: /goto <connected player id>");
            return 0;
        }

        new Float:x, Float:y, Float:z, Float:angle;
        GetPlayerPos(targetid, x, y, z);
        GetPlayerFacingAngle(targetid, angle);
        SetPlayerVirtualWorld(playerid, GetPlayerVirtualWorld(targetid));
        SetPlayerInterior(playerid, GetPlayerInterior(targetid));
        SetPlayerPos(playerid, x + 1.0, y, z);
        SetPlayerFacingAngle(playerid, angle);
        SendClientMessage(playerid, R3_PROBE_COLOUR, "Teleported to the selected player.");
        return 0;
    }

    return 0;
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
    if (!strcmp(text, R3_PROBE_ENTITY_REQUEST, false))
    {
        new Float:x, Float:y, Float:z;
        new Float:offset = 8.0;
        new objectid, vehicleid, pickupid;
        new gangzoneid;
        new message[96];
        GetPlayerPos(playerid, x, y, z);
        for (new otherid = 0; otherid < MAX_PLAYERS; otherid++)
        {
            if (otherid == playerid || !IsPlayerConnected(otherid))
            {
                continue;
            }
            SetPlayerVirtualWorld(otherid, GetPlayerVirtualWorld(playerid));
            SetPlayerInterior(otherid, GetPlayerInterior(playerid));
            SetPlayerPos(otherid, x + offset, y, z);
            offset += 2.0;
        }
        objectid = CreateObject(19300, x + 2.0, y, z, 0.0, 0.0, 0.0);
        vehicleid = CreateVehicle(411, x + 4.0, y, z, 0.0, 1, 1, -1);
        pickupid = CreatePickup(1239, 1, x + 1.0, y, z, -1);
        gangzoneid = GangZoneCreate(x - 3.0, y - 3.0, x + 3.0, y + 3.0);
        GangZoneShowForPlayer(playerid, gangzoneid, R3_PROBE_COLOUR);
        format(message, sizeof message, "R3_SDK_ENTITY_IDS_%d,%d,%d,%d", objectid, vehicleid, pickupid, gangzoneid);
        SendClientMessage(playerid, R3_PROBE_COLOUR, message);
        printf("[r3_network_probe] R3_ENTITIES_SENT playerid=%d object=%d vehicle=%d pickup=%d gangzone=%d", playerid, objectid, vehicleid, pickupid, gangzoneid);
        return 0;
    }
    return 1;
}
