#define FILTERSCRIPT

#include <a_samp>

#define DL_PROBE_OUTBOUND "DL_SDK_OUTBOUND_20260812"
#define DL_PROBE_INCOMING "DL_SDK_INCOMING_20260812"
#define DL_PROBE_DIALOG_REQUEST "DL_SDK_DIALOG_REQUEST_20260812"
#define DL_PROBE_ENTITY_REQUEST "DL_SDK_ENTITY_REQUEST_20260813"
#define DL_PROBE_LOCAL_DRIVER_REQUEST "DL_SDK_LOCAL_DRIVER_REQUEST"
#define DL_PROBE_LOCAL_PASSENGER_REQUEST "DL_SDK_LOCAL_PASSENGER_REQUEST"
#define DL_PROBE_LOCAL_TRAILER_REQUEST "DL_SDK_LOCAL_TRAILER_REQUEST"
#define DL_PROBE_VEHICLE_CLEANUP "DL_SDK_VEHICLE_CLEANUP"
#define DL_PROBE_COLOUR 0x6FCF97FF
#define DL_PROBE_DIALOG_ID 25000
#define DL_PROBE_NPC_NAME "DLProbeBot"
#define DL_PROBE_X 1880.0
#define DL_PROBE_Y -2490.0
#define DL_PROBE_Z 13.5
#define DL_PROBE_ANGLE 90.0
#define DL_PROBE_TRUCK_X 1880.0
#define DL_PROBE_TRUCK_Y -2470.0
#define DL_PROBE_TRUCK_ANGLE 0.0
#define DL_PROBE_TRAILER_Y -2482.0

new gProbeVehicle[MAX_PLAYERS];
new gProbeTruck[MAX_PLAYERS];
new gProbeTrailer[MAX_PLAYERS];

public OnFilterScriptInit()
{
    for (new playerid = 0; playerid < MAX_PLAYERS; playerid++)
    {
        ResetProbeVehicles(playerid);
    }
    print("[dl_network_probe] ready");
    return 1;
}

public OnPlayerRequestClass(playerid, classid)
{
    if (!IsProbeNpc(playerid)) return 1;
    SetSpawnInfo(playerid, 0, 61, 1958.3783, 1343.1572, 15.3746, 90.0, -1, -1, -1, -1, -1, -1);
    return 0;
}

public OnPlayerSpawn(playerid)
{
    if (IsProbeNpc(playerid)) SetPlayerColor(playerid, DL_PROBE_COLOUR);
    return 1;
}

public OnPlayerDisconnect(playerid, reason)
{
    CleanupProbeVehicles(playerid);
    return 1;
}

public OnPlayerCommandText(playerid, cmdtext[])
{
    if (strfind(cmdtext, "/goto ", true) == 0)
    {
        new targetid = strval(cmdtext[6]);
        if (!IsPlayerConnected(targetid) || targetid == playerid)
        {
            SendClientMessage(playerid, DL_PROBE_COLOUR, "Usage: /goto <connected player id>");
            return 0;
        }

        new Float:x, Float:y, Float:z, Float:angle;
        GetPlayerPos(targetid, x, y, z);
        GetPlayerFacingAngle(targetid, angle);
        SetPlayerVirtualWorld(playerid, GetPlayerVirtualWorld(targetid));
        SetPlayerInterior(playerid, GetPlayerInterior(targetid));
        SetPlayerPos(playerid, x + 1.0, y, z);
        SetPlayerFacingAngle(playerid, angle);
        SendClientMessage(playerid, DL_PROBE_COLOUR, "Teleported to the selected player.");
        return 0;
    }

    return 0;
}

public OnPlayerText(playerid, text[])
{

    if (!strcmp(text, DL_PROBE_OUTBOUND, false))
    {
        printf("[dl_network_probe] DL_OUTBOUND_OK playerid=%d", playerid);
        SendClientMessage(playerid, DL_PROBE_COLOUR, DL_PROBE_INCOMING);
        printf("[dl_network_probe] DL_INCOMING_SENT playerid=%d", playerid);
        return 0;
    }
    if (!strcmp(text, DL_PROBE_DIALOG_REQUEST, false))
    {
        ShowPlayerDialog(
            playerid,
            DL_PROBE_DIALOG_ID,
            DIALOG_STYLE_MSGBOX,
            "DL dialog cache probe",
            "Leave this dialog open while the SDK verifies its active flag.",
            "OK",
            ""
        );
        printf("[dl_network_probe] DL_DIALOG_SENT playerid=%d", playerid);
        return 0;
    }
    if (!strcmp(text, DL_PROBE_ENTITY_REQUEST, false))
    {
        new Float:x, Float:y, Float:z;
        new Float:offset = 8.0;
        new objectid, vehicleid, pickupid;
        new gangzoneid;
        new message[96];
        x = DL_PROBE_X;
        y = DL_PROBE_Y;
        z = DL_PROBE_Z;
        SetPlayerVirtualWorld(playerid, 0);
        SetPlayerInterior(playerid, 0);
        SetPlayerPos(playerid, x, y, z);
        SetPlayerFacingAngle(playerid, DL_PROBE_ANGLE);
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
        vehicleid = CreateVehicle(411, x + 4.0, y, z, DL_PROBE_ANGLE, 1, 1, -1);
        gProbeVehicle[playerid] = vehicleid;
        pickupid = CreatePickup(1239, 1, x + 1.0, y, z, -1);
        gangzoneid = GangZoneCreate(x - 3.0, y - 3.0, x + 3.0, y + 3.0);
        GangZoneShowForPlayer(playerid, gangzoneid, DL_PROBE_COLOUR);
        format(message, sizeof message, "DL_SDK_ENTITY_IDS_%d,%d,%d,%d", objectid, vehicleid, pickupid, gangzoneid);
        SendClientMessage(playerid, DL_PROBE_COLOUR, message);
        printf("[dl_network_probe] DL_ENTITIES_SENT playerid=%d object=%d vehicle=%d pickup=%d gangzone=%d", playerid, objectid, vehicleid, pickupid, gangzoneid);
        return 0;
    }
    if (!strcmp(text, DL_PROBE_LOCAL_DRIVER_REQUEST, false))
    {
        new message[64];
        if (!ProbeVehicleIsValid(gProbeVehicle[playerid])) return 0;
        PutPlayerInVehicle(playerid, gProbeVehicle[playerid], 0);
        format(message, sizeof message, "DL_SDK_LOCAL_DRIVER_READY_%d", gProbeVehicle[playerid]);
        SendClientMessage(playerid, DL_PROBE_COLOUR, message);
        return 0;
    }
    if (!strcmp(text, DL_PROBE_LOCAL_PASSENGER_REQUEST, false))
    {
        new message[64];
        if (!ProbeVehicleIsValid(gProbeVehicle[playerid])) return 0;
        PutPlayerInVehicle(playerid, gProbeVehicle[playerid], 1);
        format(message, sizeof message, "DL_SDK_LOCAL_PASSENGER_READY_%d", gProbeVehicle[playerid]);
        SendClientMessage(playerid, DL_PROBE_COLOUR, message);
        return 0;
    }
    if (!strcmp(text, DL_PROBE_LOCAL_TRAILER_REQUEST, false))
    {
        new message[64];
        SetPlayerPos(playerid, DL_PROBE_TRUCK_X, DL_PROBE_TRUCK_Y, DL_PROBE_Z);
        SetPlayerFacingAngle(playerid, DL_PROBE_TRUCK_ANGLE);
        gProbeTruck[playerid] = CreateVehicle(515, DL_PROBE_TRUCK_X, DL_PROBE_TRUCK_Y, DL_PROBE_Z, DL_PROBE_TRUCK_ANGLE, 1, 1, -1);
        gProbeTrailer[playerid] = CreateVehicle(435, DL_PROBE_TRUCK_X, DL_PROBE_TRAILER_Y, DL_PROBE_Z, DL_PROBE_TRUCK_ANGLE, 1, 1, -1);
        AttachTrailerToVehicle(gProbeTrailer[playerid], gProbeTruck[playerid]);
        PutPlayerInVehicle(playerid, gProbeTruck[playerid], 0);
        format(message, sizeof message, "DL_SDK_LOCAL_TRAILER_READY_%d,%d", gProbeTruck[playerid], gProbeTrailer[playerid]);
        SendClientMessage(playerid, DL_PROBE_COLOUR, message);
        return 0;
    }
    if (!strcmp(text, DL_PROBE_VEHICLE_CLEANUP, false))
    {
        CleanupProbeVehicles(playerid);
        SendClientMessage(playerid, DL_PROBE_COLOUR, "DL_SDK_VEHICLE_CLEANUP_READY");
        return 0;
    }
    return 1;
}

stock CleanupProbeVehicles(playerid)
{
    RemovePlayerFromVehicle(playerid);
    if (ProbeVehicleIsValid(gProbeVehicle[playerid])) DestroyVehicle(gProbeVehicle[playerid]);
    if (ProbeVehicleIsValid(gProbeTruck[playerid])) DestroyVehicle(gProbeTruck[playerid]);
    if (ProbeVehicleIsValid(gProbeTrailer[playerid])) DestroyVehicle(gProbeTrailer[playerid]);
    ResetProbeVehicles(playerid);
}

stock ResetProbeVehicles(playerid)
{
    gProbeVehicle[playerid] = INVALID_VEHICLE_ID;
    gProbeTruck[playerid] = INVALID_VEHICLE_ID;
    gProbeTrailer[playerid] = INVALID_VEHICLE_ID;
}

stock IsProbeNpc(playerid)
{
    if (!IsPlayerNPC(playerid)) return 0;
    new name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof name);
    return !strcmp(name, DL_PROBE_NPC_NAME, true);
}

stock ProbeVehicleIsValid(vehicleid)
{
    return vehicleid != INVALID_VEHICLE_ID && GetVehicleModel(vehicleid) != 0;
}
