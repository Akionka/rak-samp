#define FILTERSCRIPT

#include <a_samp>

#define R3_PROBE_OUTBOUND "R3_SDK_OUTBOUND_20260812"
#define R3_PROBE_INCOMING "R3_SDK_INCOMING_20260812"
#define R3_PROBE_DIALOG_REQUEST "R3_SDK_DIALOG_REQUEST_20260812"
#define R3_PROBE_ENTITY_REQUEST "R3_SDK_ENTITY_REQUEST_20260813"
#define R3_PROBE_LOCAL_DRIVER_REQUEST "R3_SDK_LOCAL_DRIVER_REQUEST"
#define R3_PROBE_LOCAL_PASSENGER_REQUEST "R3_SDK_LOCAL_PASSENGER_REQUEST"
#define R3_PROBE_LOCAL_TRAILER_REQUEST "R3_SDK_LOCAL_TRAILER_REQUEST"
#define R3_PROBE_VEHICLE_CLEANUP "R3_SDK_VEHICLE_CLEANUP"
#define R3_PROBE_COLOUR 0x6FCF97FF
#define R3_PROBE_DIALOG_ID 25000
#define R3_PROBE_NPC_NAME "R3ProbeBot"
#define R3_PROBE_X 1880.0
#define R3_PROBE_Y -2490.0
#define R3_PROBE_Z 13.5
#define R3_PROBE_ANGLE 90.0
#define R3_PROBE_TRUCK_X 1880.0
#define R3_PROBE_TRUCK_Y -2470.0
#define R3_PROBE_TRUCK_ANGLE 0.0
#define R3_PROBE_TRAILER_Y -2482.0

new gProbeVehicle[MAX_PLAYERS];
new gProbeTruck[MAX_PLAYERS];
new gProbeTrailer[MAX_PLAYERS];

public OnFilterScriptInit()
{
    for (new playerid = 0; playerid < MAX_PLAYERS; playerid++)
    {
        ResetProbeVehicles(playerid);
    }
    print("[r3_network_probe] ready");
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
    if (IsProbeNpc(playerid)) SetPlayerColor(playerid, R3_PROBE_COLOUR);
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
        x = R3_PROBE_X;
        y = R3_PROBE_Y;
        z = R3_PROBE_Z;
        SetPlayerVirtualWorld(playerid, 0);
        SetPlayerInterior(playerid, 0);
        SetPlayerPos(playerid, x, y, z);
        SetPlayerFacingAngle(playerid, R3_PROBE_ANGLE);
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
        vehicleid = CreateVehicle(411, x + 4.0, y, z, R3_PROBE_ANGLE, 1, 1, -1);
        gProbeVehicle[playerid] = vehicleid;
        pickupid = CreatePickup(1239, 1, x + 1.0, y, z, -1);
        gangzoneid = GangZoneCreate(x - 3.0, y - 3.0, x + 3.0, y + 3.0);
        GangZoneShowForPlayer(playerid, gangzoneid, R3_PROBE_COLOUR);
        format(message, sizeof message, "R3_SDK_ENTITY_IDS_%d,%d,%d,%d", objectid, vehicleid, pickupid, gangzoneid);
        SendClientMessage(playerid, R3_PROBE_COLOUR, message);
        printf("[r3_network_probe] R3_ENTITIES_SENT playerid=%d object=%d vehicle=%d pickup=%d gangzone=%d", playerid, objectid, vehicleid, pickupid, gangzoneid);
        return 0;
    }
    if (!strcmp(text, R3_PROBE_LOCAL_DRIVER_REQUEST, false))
    {
        new message[64];
        if (!ProbeVehicleIsValid(gProbeVehicle[playerid])) return 0;
        PutPlayerInVehicle(playerid, gProbeVehicle[playerid], 0);
        format(message, sizeof message, "R3_SDK_LOCAL_DRIVER_READY_%d", gProbeVehicle[playerid]);
        SendClientMessage(playerid, R3_PROBE_COLOUR, message);
        return 0;
    }
    if (!strcmp(text, R3_PROBE_LOCAL_PASSENGER_REQUEST, false))
    {
        new message[64];
        if (!ProbeVehicleIsValid(gProbeVehicle[playerid])) return 0;
        PutPlayerInVehicle(playerid, gProbeVehicle[playerid], 1);
        format(message, sizeof message, "R3_SDK_LOCAL_PASSENGER_READY_%d", gProbeVehicle[playerid]);
        SendClientMessage(playerid, R3_PROBE_COLOUR, message);
        return 0;
    }
    if (!strcmp(text, R3_PROBE_LOCAL_TRAILER_REQUEST, false))
    {
        new message[64];
        SetPlayerPos(playerid, R3_PROBE_TRUCK_X, R3_PROBE_TRUCK_Y, R3_PROBE_Z);
        SetPlayerFacingAngle(playerid, R3_PROBE_TRUCK_ANGLE);
        gProbeTruck[playerid] = CreateVehicle(515, R3_PROBE_TRUCK_X, R3_PROBE_TRUCK_Y, R3_PROBE_Z, R3_PROBE_TRUCK_ANGLE, 1, 1, -1);
        gProbeTrailer[playerid] = CreateVehicle(435, R3_PROBE_TRUCK_X, R3_PROBE_TRAILER_Y, R3_PROBE_Z, R3_PROBE_TRUCK_ANGLE, 1, 1, -1);
        AttachTrailerToVehicle(gProbeTrailer[playerid], gProbeTruck[playerid]);
        PutPlayerInVehicle(playerid, gProbeTruck[playerid], 0);
        format(message, sizeof message, "R3_SDK_LOCAL_TRAILER_READY_%d,%d", gProbeTruck[playerid], gProbeTrailer[playerid]);
        SendClientMessage(playerid, R3_PROBE_COLOUR, message);
        return 0;
    }
    if (!strcmp(text, R3_PROBE_VEHICLE_CLEANUP, false))
    {
        CleanupProbeVehicles(playerid);
        SendClientMessage(playerid, R3_PROBE_COLOUR, "R3_SDK_VEHICLE_CLEANUP_READY");
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
    return !strcmp(name, R3_PROBE_NPC_NAME, true);
}

stock ProbeVehicleIsValid(vehicleid)
{
    return vehicleid != INVALID_VEHICLE_ID && GetVehicleModel(vehicleid) != 0;
}
