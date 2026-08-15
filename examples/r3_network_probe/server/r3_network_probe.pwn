#define FILTERSCRIPT

#include <a_samp>

#define R3_PROBE_OUTBOUND "R3_SDK_OUTBOUND_20260812"
#define R3_PROBE_INCOMING "R3_SDK_INCOMING_20260812"
#define R3_PROBE_DIALOG_REQUEST "R3_SDK_DIALOG_REQUEST_20260812"
#define R3_PROBE_ENTITY_REQUEST "R3_SDK_ENTITY_REQUEST_20260813"
#define R3_PROBE_LOCAL_DRIVER_REQUEST "R3_SDK_LOCAL_DRIVER_REQUEST"
#define R3_PROBE_LOCAL_PASSENGER_REQUEST "R3_SDK_LOCAL_PASSENGER_REQUEST"
#define R3_PROBE_LOCAL_TRAILER_REQUEST "R3_SDK_LOCAL_TRAILER_REQUEST"
#define R3_PROBE_REMOTE_DRIVER_REQUEST "R3_SDK_REMOTE_DRIVER_REQUEST"
#define R3_PROBE_REMOTE_PASSENGER_REQUEST "R3_SDK_REMOTE_PASSENGER_REQUEST"
#define R3_PROBE_REMOTE_TRAILER_REQUEST "R3_SDK_REMOTE_TRAILER_REQUEST"
#define R3_PROBE_VEHICLE_CLEANUP "R3_SDK_VEHICLE_CLEANUP"
#define R3_PROBE_COLOUR 0x6FCF97FF
#define R3_PROBE_DIALOG_ID 25000

new gProbeVehicle[MAX_PLAYERS];
new gProbeTruck[MAX_PLAYERS];
new gProbeTrailer[MAX_PLAYERS];
new gRemoteVehicle[MAX_PLAYERS];
new gRemoteTruck[MAX_PLAYERS];
new gRemoteTrailer[MAX_PLAYERS];
new gRemotePlayer[MAX_PLAYERS];

public OnFilterScriptInit()
{
    for (new playerid = 0; playerid < MAX_PLAYERS; playerid++)
    {
        ResetProbeVehicles(playerid);
    }
    print("[r3_network_probe] ready");
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
        new Float:x, Float:y, Float:z;
        new message[64];
        GetPlayerPos(playerid, x, y, z);
        gProbeTruck[playerid] = CreateVehicle(515, x + 6.0, y, z, 0.0, 1, 1, -1);
        gProbeTrailer[playerid] = CreateVehicle(435, x + 10.0, y, z, 0.0, 1, 1, -1);
        AttachTrailerToVehicle(gProbeTrailer[playerid], gProbeTruck[playerid]);
        PutPlayerInVehicle(playerid, gProbeTruck[playerid], 0);
        format(message, sizeof message, "R3_SDK_LOCAL_TRAILER_READY_%d,%d", gProbeTruck[playerid], gProbeTrailer[playerid]);
        SendClientMessage(playerid, R3_PROBE_COLOUR, message);
        return 0;
    }
    if (!strcmp(text, R3_PROBE_REMOTE_DRIVER_REQUEST, false))
    {
        new Float:x, Float:y, Float:z;
        new message[64];
        gRemotePlayer[playerid] = FindOtherPlayer(playerid);
        if (gRemotePlayer[playerid] == INVALID_PLAYER_ID) return 0;
        GetPlayerPos(playerid, x, y, z);
        gRemoteVehicle[playerid] = CreateVehicle(560, x + 8.0, y, z, 0.0, 1, 1, -1);
        PutPlayerInVehicle(gRemotePlayer[playerid], gRemoteVehicle[playerid], 0);
        format(message, sizeof message, "R3_SDK_REMOTE_DRIVER_READY_%d,%d", gRemotePlayer[playerid], gRemoteVehicle[playerid]);
        SendClientMessage(playerid, R3_PROBE_COLOUR, message);
        return 0;
    }
    if (!strcmp(text, R3_PROBE_REMOTE_PASSENGER_REQUEST, false))
    {
        new message[64];
        if (gRemotePlayer[playerid] == INVALID_PLAYER_ID || !ProbeVehicleIsValid(gRemoteVehicle[playerid])) return 0;
        PutPlayerInVehicle(gRemotePlayer[playerid], gRemoteVehicle[playerid], 1);
        format(message, sizeof message, "R3_SDK_REMOTE_PASSENGER_READY_%d,%d", gRemotePlayer[playerid], gRemoteVehicle[playerid]);
        SendClientMessage(playerid, R3_PROBE_COLOUR, message);
        return 0;
    }
    if (!strcmp(text, R3_PROBE_REMOTE_TRAILER_REQUEST, false))
    {
        new Float:x, Float:y, Float:z;
        new message[80];
        if (gRemotePlayer[playerid] == INVALID_PLAYER_ID) return 0;
        GetPlayerPos(playerid, x, y, z);
        gRemoteTruck[playerid] = CreateVehicle(515, x + 10.0, y, z, 0.0, 1, 1, -1);
        gRemoteTrailer[playerid] = CreateVehicle(435, x + 14.0, y, z, 0.0, 1, 1, -1);
        AttachTrailerToVehicle(gRemoteTrailer[playerid], gRemoteTruck[playerid]);
        PutPlayerInVehicle(gRemotePlayer[playerid], gRemoteTruck[playerid], 0);
        format(message, sizeof message, "R3_SDK_REMOTE_TRAILER_READY_%d,%d,%d", gRemotePlayer[playerid], gRemoteTruck[playerid], gRemoteTrailer[playerid]);
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

stock FindOtherPlayer(playerid)
{
    for (new otherid = 0; otherid < MAX_PLAYERS; otherid++)
    {
        if (otherid != playerid && IsPlayerConnected(otherid)) return otherid;
    }
    return INVALID_PLAYER_ID;
}

stock CleanupProbeVehicles(playerid)
{
    RemovePlayerFromVehicle(playerid);
    if (gRemotePlayer[playerid] != INVALID_PLAYER_ID && IsPlayerConnected(gRemotePlayer[playerid]))
    {
        RemovePlayerFromVehicle(gRemotePlayer[playerid]);
    }
    if (ProbeVehicleIsValid(gProbeVehicle[playerid])) DestroyVehicle(gProbeVehicle[playerid]);
    if (ProbeVehicleIsValid(gProbeTruck[playerid])) DestroyVehicle(gProbeTruck[playerid]);
    if (ProbeVehicleIsValid(gProbeTrailer[playerid])) DestroyVehicle(gProbeTrailer[playerid]);
    if (ProbeVehicleIsValid(gRemoteVehicle[playerid])) DestroyVehicle(gRemoteVehicle[playerid]);
    if (ProbeVehicleIsValid(gRemoteTruck[playerid])) DestroyVehicle(gRemoteTruck[playerid]);
    if (ProbeVehicleIsValid(gRemoteTrailer[playerid])) DestroyVehicle(gRemoteTrailer[playerid]);
    ResetProbeVehicles(playerid);
}

stock ResetProbeVehicles(playerid)
{
    gProbeVehicle[playerid] = INVALID_VEHICLE_ID;
    gProbeTruck[playerid] = INVALID_VEHICLE_ID;
    gProbeTrailer[playerid] = INVALID_VEHICLE_ID;
    gRemoteVehicle[playerid] = INVALID_VEHICLE_ID;
    gRemoteTruck[playerid] = INVALID_VEHICLE_ID;
    gRemoteTrailer[playerid] = INVALID_VEHICLE_ID;
    gRemotePlayer[playerid] = INVALID_PLAYER_ID;
}

stock ProbeVehicleIsValid(vehicleid)
{
    return vehicleid != INVALID_VEHICLE_ID && GetVehicleModel(vehicleid) != 0;
}
