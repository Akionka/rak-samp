#include <a_npc>

forward MoveProbeBot();

new bool:gMoveForward;

main() {}

public OnNPCSpawn()
{
    SetTimer("MoveProbeBot", 750, true);
    return 1;
}

public MoveProbeBot()
{
    new Float:x, Float:y, Float:z;
    GetMyPos(x, y, z);
    if (gMoveForward)
    {
        x += 0.75;
        SetMyFacingAngle(90.0);
    }
    else
    {
        x -= 0.75;
        SetMyFacingAngle(270.0);
    }
    gMoveForward = !gMoveForward;
    SetMyPos(x, y, z);
    return 1;
}
