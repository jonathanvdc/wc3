//! Checked concrete polymorphic record layouts.
use super::*;
/// Supported concrete layouts of polymorphic records.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum Polymorphic<'data> {
    /// Native `IKGeneralBoneChain`.
    IKGeneralBoneChain(Box<IKGeneralBoneChain>),
    /// Native `TriggerEventConstantFlash`.
    TriggerEventConstantFlash(Box<TriggerEventConstantFlash>),
    /// Native `TriggerEventFadeOtherPlayers`.
    TriggerEventFadeOtherPlayers(Box<TriggerEventFadeOtherPlayers>),
    /// Native `TriggerEventStoreUIIntro`.
    TriggerEventStoreUIIntro(Box<TriggerEventStoreUIIntro>),
    /// Native `TriggerEventAttachSpawnedActor`.
    TriggerEventAttachSpawnedActor(Box<TriggerEventAttachSpawnedActor>),
    /// Native `TexAnimTranslationCurve`.
    TexAnimTranslationCurve(Box<TexAnimTranslationCurve>),
    /// Native `TriggerEventRelaxCloth`.
    TriggerEventRelaxCloth(Box<TriggerEventRelaxCloth>),
    /// Native `TriggerEventSetAudioState`.
    TriggerEventSetAudioState(Box<TriggerEventSetAudioState>),
    /// Native `TriggerEventObjectTurnToFace`.
    TriggerEventObjectTurnToFace(Box<TriggerEventObjectTurnToFace>),
    /// Native `TriggerEventChangeShaderMapOverrideDuration`.
    TriggerEventChangeShaderMapOverrideDuration(Box<TriggerEventChangeShaderMapOverrideDuration>),
    /// Native `TriggerEventShadow`.
    TriggerEventShadow(Box<TriggerEventShadow>),
    /// Native `TriggerEventIGCLightMode`.
    TriggerEventIGCLightMode(Box<TriggerEventIGCLightMode>),
    /// Native `IKBoneChainBase`.
    IKBoneChainBase(Box<IKBoneChainBase>),
    /// Native `TriggerEventHidePoIUI`.
    TriggerEventHidePoIUI(Box<TriggerEventHidePoIUI>),
    /// Native `TriggerEventShadowDuration`.
    TriggerEventShadowDuration(Box<TriggerEventShadowDuration>),
    /// Native `TriggerEventCameraChangeNearFarPlanes`.
    TriggerEventCameraChangeNearFarPlanes(Box<TriggerEventCameraChangeNearFarPlanes>),
    /// Native `TriggerEventAddObject`.
    TriggerEventAddObject(Box<TriggerEventAddObject<'data>>),
    /// Native `TriggerEventDeleteObject`.
    TriggerEventDeleteObject(Box<TriggerEventDeleteObject>),
    /// Native `TriggerEventActorMovement`.
    TriggerEventActorMovement(Box<TriggerEventActorMovement>),
    /// Native `SpeedTreeWind8`.
    SpeedTreeWind8(Box<SpeedTreeWind8>),
    /// Native `SpeedTreeWind9`.
    SpeedTreeWind9(Box<SpeedTreeWind9>),
    /// Native `TriggerEventFoley`.
    TriggerEventFoley(Box<TriggerEventFoley>),
    /// Native `TexAnim`.
    TexAnim(Box<TexAnim>),
    /// Native `TriggerEventMusic`.
    TriggerEventMusic(Box<TriggerEventMusic>),
    /// Native `TriggerEventVoice`.
    TriggerEventVoice(Box<TriggerEventVoice>),
    /// Native `TexAnimFrameByFrame`.
    TexAnimFrameByFrame(Box<TexAnimFrameByFrame>),
    /// Native `TriggerEventDisableRagdollAnimationOverride`.
    TriggerEventDisableRagdollAnimationOverride(Box<TriggerEventDisableRagdollAnimationOverride>),
    /// Native `TriggerEventChangeClothParameters`.
    TriggerEventChangeClothParameters(Box<TriggerEventChangeClothParameters>),
    /// Native `TriggerEventChangeShaderMapOverride`.
    TriggerEventChangeShaderMapOverride(Box<TriggerEventChangeShaderMapOverride>),
    /// Native `TriggerEventHitFlashDuration`.
    TriggerEventHitFlashDuration(Box<TriggerEventHitFlashDuration>),
    /// Native `TriggerEventFrostBreath`.
    TriggerEventFrostBreath(Box<TriggerEventFrostBreath>),
    /// Native `TriggerEventDeleteAllObjects`.
    TriggerEventDeleteAllObjects(Box<TriggerEventDeleteAllObjects>),
    /// Native `TriggerEventCutsceneUIKeyFrame`.
    TriggerEventCutsceneUIKeyFrame(Box<TriggerEventCutsceneUIKeyFrame<'data>>),
    /// Native `TriggerEventAddBaseMountLoadout`.
    TriggerEventAddBaseMountLoadout(Box<TriggerEventAddBaseMountLoadout>),
    /// Native `TriggerEventSubObjectVisibility`.
    TriggerEventSubObjectVisibility(Box<TriggerEventSubObjectVisibility>),
    /// Native `TriggerEventHideWarpUI`.
    TriggerEventHideWarpUI(Box<TriggerEventHideWarpUI>),
    /// Native `TriggerEventFlappyJawOverrideDuration`.
    TriggerEventFlappyJawOverrideDuration(Box<TriggerEventFlappyJawOverrideDuration>),
    /// Native `TriggerEventAddTrail`.
    TriggerEventAddTrail(Box<TriggerEventAddTrail>),
    /// Native `TriggerEventCameraMoveToCameraMarker`.
    TriggerEventCameraMoveToCameraMarker(Box<TriggerEventCameraMoveToCameraMarker>),
    /// Native `TriggerEventDisableConstraint`.
    TriggerEventDisableConstraint(Box<TriggerEventDisableConstraint>),
    /// Native `TriggerEventCancelLook`.
    TriggerEventCancelLook(Box<TriggerEventCancelLook>),
    /// Native `TriggerEventFootstep`.
    TriggerEventFootstep(Box<TriggerEventFootstep>),
    /// Native `TriggerEventCameraTransitionToObserver`.
    TriggerEventCameraTransitionToObserver(Box<TriggerEventCameraTransitionToObserver>),
    /// Native `TriggerEventHideCaptionUI`.
    TriggerEventHideCaptionUI(Box<TriggerEventHideCaptionUI>),
    /// Native `TriggerEventAddEffectGroup`.
    TriggerEventAddEffectGroup(Box<TriggerEventAddEffectGroup>),
    /// Native `TriggerEventActorPlayAnimation`.
    TriggerEventActorPlayAnimation(Box<TriggerEventActorPlayAnimation>),
    /// Native `TriggerEventFadeRemotePlayers`.
    TriggerEventFadeRemotePlayers(Box<TriggerEventFadeRemotePlayers>),
    /// Native `TriggerEventIGCHideGroup`.
    TriggerEventIGCHideGroup(Box<TriggerEventIGCHideGroup>),
    /// Native `TriggerEventHitFlash`.
    TriggerEventHitFlash(Box<TriggerEventHitFlash>),
    /// Native `TriggerEventFootstepDrag`.
    TriggerEventFootstepDrag(Box<TriggerEventFootstepDrag>),
    /// Native `TriggerEventFootstepLift`.
    TriggerEventFootstepLift(Box<TriggerEventFootstepLift>),
    /// Native `TriggerEventDisableCollisionMessages`.
    TriggerEventDisableCollisionMessages(Box<TriggerEventDisableCollisionMessages>),
    /// Native `TriggerEventSequenceEnded`.
    TriggerEventSequenceEnded(Box<TriggerEventSequenceEnded>),
    /// Native `IKLegBoneChain`.
    IKLegBoneChain(Box<IKLegBoneChain>),
    /// Native `TriggerEventChangePhysicsParameters`.
    TriggerEventChangePhysicsParameters(Box<TriggerEventChangePhysicsParameters>),
    /// Native `TriggerEventTransparency`.
    TriggerEventTransparency(Box<TriggerEventTransparency>),
    /// Native `TriggerEventObjectLookAt`.
    TriggerEventObjectLookAt(Box<TriggerEventObjectLookAt>),
    /// Native `TriggerEventMsg`.
    TriggerEventMsg(Box<TriggerEventMsg>),
    /// Native `TriggerEvent`.
    TriggerEvent(Box<TriggerEvent>),
    /// Native `ActorGizmoData`.
    ActorGizmoData(Box<ActorGizmoData>),
    /// Native `TriggerEventAddRope`.
    TriggerEventAddRope(Box<TriggerEventAddRope>),
    /// Native `TriggerEventSetWeaponSheathed`.
    TriggerEventSetWeaponSheathed(Box<TriggerEventSetWeaponSheathed>),
    /// Native `TriggerEventCameraStayOnSpline`.
    TriggerEventCameraStayOnSpline(Box<TriggerEventCameraStayOnSpline>),
    /// Native `TriggerEventSetWeaponVisibility`.
    TriggerEventSetWeaponVisibility(Box<TriggerEventSetWeaponVisibility>),
    /// Native `TriggerEventCameraChangeFieldOfView`.
    TriggerEventCameraChangeFieldOfView(Box<TriggerEventCameraChangeFieldOfView>),
    /// Native `TriggerEventChangeLookDuration`.
    TriggerEventChangeLookDuration(Box<TriggerEventChangeLookDuration>),
    /// Native `TriggerEventSetOtherScenesVisibility`.
    TriggerEventSetOtherScenesVisibility(Box<TriggerEventSetOtherScenesVisibility>),
    /// Native `TriggerEventShowWarpUI`.
    TriggerEventShowWarpUI(Box<TriggerEventShowWarpUI>),
    /// Native `TriggerEventShowCaptionUI`.
    TriggerEventShowCaptionUI(Box<TriggerEventShowCaptionUI>),
    /// Native `TriggerEventFadeMonsters`.
    TriggerEventFadeMonsters(Box<TriggerEventFadeMonsters>),
    /// Native `SpeedTreeWind`.
    SpeedTreeWind(Box<SpeedTreeWind>),
    /// Native `TriggerEventFadePlayers`.
    TriggerEventFadePlayers(Box<TriggerEventFadePlayers>),
    /// Native `TriggerEventSetWeaponVisibilityDuration`.
    TriggerEventSetWeaponVisibilityDuration(Box<TriggerEventSetWeaponVisibilityDuration>),
    /// Native `TriggerEventSpawnObject`.
    TriggerEventSpawnObject(Box<TriggerEventSpawnObject<'data>>),
    /// Native `TriggerEventCameraChangeTarget`.
    TriggerEventCameraChangeTarget(Box<TriggerEventCameraChangeTarget>),
    /// Native `TriggerEventEnableRagdollAnimationOverride`.
    TriggerEventEnableRagdollAnimationOverride(Box<TriggerEventEnableRagdollAnimationOverride>),
    /// Native `SpeedTreeWindBase`.
    SpeedTreeWindBase(Box<SpeedTreeWindBase>),
    /// Native `TriggerEventDetachObject`.
    TriggerEventDetachObject(Box<TriggerEventDetachObject>),
    /// Native `TriggerEventRestoreCamera`.
    TriggerEventRestoreCamera(Box<TriggerEventRestoreCamera>),
    /// Native `EffectGroupKeyData`.
    EffectGroupKeyData(Box<EffectGroupKeyData>),
    /// Native `TriggerEventSetHelmVisibility`.
    TriggerEventSetHelmVisibility(Box<TriggerEventSetHelmVisibility>),
    /// Native `TriggerEventShowPoIUI`.
    TriggerEventShowPoIUI(Box<TriggerEventShowPoIUI>),
    /// Native `TriggerEventObjectMoveToLocator`.
    TriggerEventObjectMoveToLocator(Box<TriggerEventObjectMoveToLocator>),
    /// Native `TriggerEventSetRTCDebugText`.
    TriggerEventSetRTCDebugText(Box<TriggerEventSetRTCDebugText<'data>>),
    /// Native `TexAnimFrameByFrameCurve`.
    TexAnimFrameByFrameCurve(Box<TexAnimFrameByFrameCurve>),
    /// Native `TriggerEventCameraChangeDepthOfField`.
    TriggerEventCameraChangeDepthOfField(Box<TriggerEventCameraChangeDepthOfField>),
    /// Native `TriggerEventSetLightingLabelOverride`.
    TriggerEventSetLightingLabelOverride(Box<TriggerEventSetLightingLabelOverride>),
    /// Native `TriggerEventObjectKeyFrame`.
    TriggerEventObjectKeyFrame(Box<TriggerEventObjectKeyFrame<'data>>),
    /// Native `TriggerEventSetOcclusionShapeVisibility`.
    TriggerEventSetOcclusionShapeVisibility(Box<TriggerEventSetOcclusionShapeVisibility>),
    /// Native `TriggerEventScreenFade`.
    TriggerEventScreenFade(Box<TriggerEventScreenFade>),
    /// Native `TriggerEventFade`.
    TriggerEventFade(Box<TriggerEventFade>),
    /// Native `TriggerEventNull`.
    TriggerEventNull(Box<TriggerEventNull>),
    /// Native `TriggerEventRoutingFunctionKeyFrames`.
    TriggerEventRoutingFunctionKeyFrames(Box<TriggerEventRoutingFunctionKeyFrames<'data>>),
    /// Native `TriggerEventPOISkip`.
    TriggerEventPOISkip(Box<TriggerEventPOISkip>),
    /// Native `TriggerEventAttachCamera`.
    TriggerEventAttachCamera(Box<TriggerEventAttachCamera>),
    /// Native `TriggerEventOutroAllObjects`.
    TriggerEventOutroAllObjects(Box<TriggerEventOutroAllObjects>),
    /// Native `TriggerEventOutroEffectGroup`.
    TriggerEventOutroEffectGroup(Box<TriggerEventOutroEffectGroup>),
    /// Native `TriggerEventSetIGCMusic`.
    TriggerEventSetIGCMusic(Box<TriggerEventSetIGCMusic>),
    /// Native `TriggerEventCameraKeyFrame`.
    TriggerEventCameraKeyFrame(Box<TriggerEventCameraKeyFrame<'data>>),
    /// Native `TriggerEventObjectTurnToOriginalFacing`.
    TriggerEventObjectTurnToOriginalFacing(Box<TriggerEventObjectTurnToOriginalFacing>),
    /// Native `TriggerEventPlayMovie`.
    TriggerEventPlayMovie(Box<TriggerEventPlayMovie>),
    /// Native `TriggerEventChangeLook`.
    TriggerEventChangeLook(Box<TriggerEventChangeLook>),
    /// Native `TriggerEventFadeGroup`.
    TriggerEventFadeGroup(Box<TriggerEventFadeGroup>),
    /// Native `TriggerEventChangeAttachmentHardpointDuration`.
    TriggerEventChangeAttachmentHardpointDuration(
        Box<TriggerEventChangeAttachmentHardpointDuration>,
    ),
    /// Native `TriggerEventFootstepTrail`.
    TriggerEventFootstepTrail(Box<TriggerEventFootstepTrail>),
    /// Native `TriggerEventSetTimeOfDay`.
    TriggerEventSetTimeOfDay(Box<TriggerEventSetTimeOfDay>),
    /// Native `TriggerEventChangeAttachmentHardpoint`.
    TriggerEventChangeAttachmentHardpoint(Box<TriggerEventChangeAttachmentHardpoint>),
    /// Native `TriggerEventObjectMoveToObject`.
    TriggerEventObjectMoveToObject(Box<TriggerEventObjectMoveToObject>),
    /// Native `Subcondition`.
    Subcondition(Box<Subcondition>),
    /// Native `TriggerEventEnableRagdoll`.
    TriggerEventEnableRagdoll(Box<TriggerEventEnableRagdoll>),
    /// Native `TexAnimTranslation`.
    TexAnimTranslation(Box<TexAnimTranslation>),
    /// Native `TriggerEventCameraFollowSpline`.
    TriggerEventCameraFollowSpline(Box<TriggerEventCameraFollowSpline>),
    /// Native `TriggerEventSetWeaponSheathedDuration`.
    TriggerEventSetWeaponSheathedDuration(Box<TriggerEventSetWeaponSheathedDuration>),
    /// Native `TriggerEventObjectFollowSpline`.
    TriggerEventObjectFollowSpline(Box<TriggerEventObjectFollowSpline>),
    /// Native `TriggerEventRestoreTimeOfDay`.
    TriggerEventRestoreTimeOfDay(Box<TriggerEventRestoreTimeOfDay>),
    /// Native `TriggerEventPlayConversationLine`.
    TriggerEventPlayConversationLine(Box<TriggerEventPlayConversationLine>),
    /// Native `TriggerEventOutroObject`.
    TriggerEventOutroObject(Box<TriggerEventOutroObject>),
    /// Native `TriggerEventListenerFollowSpline`.
    TriggerEventListenerFollowSpline(Box<TriggerEventListenerFollowSpline>),
    /// Native `TriggerEventHideWarpDestinationUI`.
    TriggerEventHideWarpDestinationUI(Box<TriggerEventHideWarpDestinationUI>),
    /// Native `TriggerEventOutroServicesDisable`.
    TriggerEventOutroServicesDisable(Box<TriggerEventOutroServicesDisable>),
    /// Native `TriggerEventRangedDuration`.
    TriggerEventRangedDuration(Box<TriggerEventRangedDuration>),
    /// Native `TriggerEventShowWarpDestinationUI`.
    TriggerEventShowWarpDestinationUI(Box<TriggerEventShowWarpDestinationUI>),
}
impl<'data> Polymorphic<'data> {
    pub(crate) fn read(
        decoder: &Decoder<'data>,
        offset: usize,
        base: u32,
    ) -> Result<Self, ReadError> {
        let hash: u32 = decoder.value(offset)?;
        let hash = if hash == 0 { base } else { hash };
        if !Self::supports_hash(base, hash) {
            return Err(ReadError::UnsupportedType(hash));
        }
        Ok(match hash {
            64253465 => {
                Self::IKGeneralBoneChain(Box::new(IKGeneralBoneChain::read(decoder, offset)?))
            }
            133533134 => Self::TriggerEventConstantFlash(Box::new(
                TriggerEventConstantFlash::read(decoder, offset)?,
            )),
            203752552 => Self::TriggerEventFadeOtherPlayers(Box::new(
                TriggerEventFadeOtherPlayers::read(decoder, offset)?,
            )),
            266386765 => Self::TriggerEventStoreUIIntro(Box::new(TriggerEventStoreUIIntro::read(
                decoder, offset,
            )?)),
            303887094 => Self::TriggerEventAttachSpawnedActor(Box::new(
                TriggerEventAttachSpawnedActor::read(decoder, offset)?,
            )),
            312499914 => Self::TexAnimTranslationCurve(Box::new(TexAnimTranslationCurve::read(
                decoder, offset,
            )?)),
            330209804 => Self::TriggerEventRelaxCloth(Box::new(TriggerEventRelaxCloth::read(
                decoder, offset,
            )?)),
            405077397 => Self::TriggerEventSetAudioState(Box::new(
                TriggerEventSetAudioState::read(decoder, offset)?,
            )),
            419152424 => Self::TriggerEventObjectTurnToFace(Box::new(
                TriggerEventObjectTurnToFace::read(decoder, offset)?,
            )),
            469541751 => Self::TriggerEventChangeShaderMapOverrideDuration(Box::new(
                TriggerEventChangeShaderMapOverrideDuration::read(decoder, offset)?,
            )),
            509045980 => {
                Self::TriggerEventShadow(Box::new(TriggerEventShadow::read(decoder, offset)?))
            }
            514913766 => Self::TriggerEventIGCLightMode(Box::new(TriggerEventIGCLightMode::read(
                decoder, offset,
            )?)),
            540107926 => Self::IKBoneChainBase(Box::new(IKBoneChainBase::read(decoder, offset)?)),
            547442486 => {
                Self::TriggerEventHidePoIUI(Box::new(TriggerEventHidePoIUI::read(decoder, offset)?))
            }
            551002818 => Self::TriggerEventShadowDuration(Box::new(
                TriggerEventShadowDuration::read(decoder, offset)?,
            )),
            563099623 => Self::TriggerEventCameraChangeNearFarPlanes(Box::new(
                TriggerEventCameraChangeNearFarPlanes::read(decoder, offset)?,
            )),
            568526390 => {
                Self::TriggerEventAddObject(Box::new(TriggerEventAddObject::read(decoder, offset)?))
            }
            570318784 => Self::TriggerEventDeleteObject(Box::new(TriggerEventDeleteObject::read(
                decoder, offset,
            )?)),
            596227098 => Self::TriggerEventActorMovement(Box::new(
                TriggerEventActorMovement::read(decoder, offset)?,
            )),
            804883275 => Self::SpeedTreeWind8(Box::new(SpeedTreeWind8::read(decoder, offset)?)),
            804883276 => Self::SpeedTreeWind9(Box::new(SpeedTreeWind9::read(decoder, offset)?)),
            911325909 => {
                Self::TriggerEventFoley(Box::new(TriggerEventFoley::read(decoder, offset)?))
            }
            911604182 => Self::TexAnim(Box::new(TexAnim::read(decoder, offset)?)),
            919850711 => {
                Self::TriggerEventMusic(Box::new(TriggerEventMusic::read(decoder, offset)?))
            }
            930297292 => {
                Self::TriggerEventVoice(Box::new(TriggerEventVoice::read(decoder, offset)?))
            }
            1011253607 => {
                Self::TexAnimFrameByFrame(Box::new(TexAnimFrameByFrame::read(decoder, offset)?))
            }
            1014113135 => Self::TriggerEventDisableRagdollAnimationOverride(Box::new(
                TriggerEventDisableRagdollAnimationOverride::read(decoder, offset)?,
            )),
            1036833962 => Self::TriggerEventChangeClothParameters(Box::new(
                TriggerEventChangeClothParameters::read(decoder, offset)?,
            )),
            1066764433 => Self::TriggerEventChangeShaderMapOverride(Box::new(
                TriggerEventChangeShaderMapOverride::read(decoder, offset)?,
            )),
            1077456687 => Self::TriggerEventHitFlashDuration(Box::new(
                TriggerEventHitFlashDuration::read(decoder, offset)?,
            )),
            1106358970 => Self::TriggerEventFrostBreath(Box::new(TriggerEventFrostBreath::read(
                decoder, offset,
            )?)),
            1109533772 => Self::TriggerEventDeleteAllObjects(Box::new(
                TriggerEventDeleteAllObjects::read(decoder, offset)?,
            )),
            1111377282 => Self::TriggerEventCutsceneUIKeyFrame(Box::new(
                TriggerEventCutsceneUIKeyFrame::read(decoder, offset)?,
            )),
            1131078565 => Self::TriggerEventAddBaseMountLoadout(Box::new(
                TriggerEventAddBaseMountLoadout::read(decoder, offset)?,
            )),
            1134541503 => Self::TriggerEventSubObjectVisibility(Box::new(
                TriggerEventSubObjectVisibility::read(decoder, offset)?,
            )),
            1144581000 => Self::TriggerEventHideWarpUI(Box::new(TriggerEventHideWarpUI::read(
                decoder, offset,
            )?)),
            1146575562 => Self::TriggerEventFlappyJawOverrideDuration(Box::new(
                TriggerEventFlappyJawOverrideDuration::read(decoder, offset)?,
            )),
            1195077723 => {
                Self::TriggerEventAddTrail(Box::new(TriggerEventAddTrail::read(decoder, offset)?))
            }
            1208547332 => Self::TriggerEventCameraMoveToCameraMarker(Box::new(
                TriggerEventCameraMoveToCameraMarker::read(decoder, offset)?,
            )),
            1288253615 => Self::TriggerEventDisableConstraint(Box::new(
                TriggerEventDisableConstraint::read(decoder, offset)?,
            )),
            1310250065 => Self::TriggerEventCancelLook(Box::new(TriggerEventCancelLook::read(
                decoder, offset,
            )?)),
            1328668874 => {
                Self::TriggerEventFootstep(Box::new(TriggerEventFootstep::read(decoder, offset)?))
            }
            1338965781 => Self::TriggerEventCameraTransitionToObserver(Box::new(
                TriggerEventCameraTransitionToObserver::read(decoder, offset)?,
            )),
            1367839516 => Self::TriggerEventHideCaptionUI(Box::new(
                TriggerEventHideCaptionUI::read(decoder, offset)?,
            )),
            1384905369 => Self::TriggerEventAddEffectGroup(Box::new(
                TriggerEventAddEffectGroup::read(decoder, offset)?,
            )),
            1487816389 => Self::TriggerEventActorPlayAnimation(Box::new(
                TriggerEventActorPlayAnimation::read(decoder, offset)?,
            )),
            1524322706 => Self::TriggerEventFadeRemotePlayers(Box::new(
                TriggerEventFadeRemotePlayers::read(decoder, offset)?,
            )),
            1538719568 => Self::TriggerEventIGCHideGroup(Box::new(TriggerEventIGCHideGroup::read(
                decoder, offset,
            )?)),
            1648188489 => {
                Self::TriggerEventHitFlash(Box::new(TriggerEventHitFlash::read(decoder, offset)?))
            }
            1670410600 => Self::TriggerEventFootstepDrag(Box::new(TriggerEventFootstepDrag::read(
                decoder, offset,
            )?)),
            1670688473 => Self::TriggerEventFootstepLift(Box::new(TriggerEventFootstepLift::read(
                decoder, offset,
            )?)),
            1687293198 => Self::TriggerEventDisableCollisionMessages(Box::new(
                TriggerEventDisableCollisionMessages::read(decoder, offset)?,
            )),
            1724776431 => Self::TriggerEventSequenceEnded(Box::new(
                TriggerEventSequenceEnded::read(decoder, offset)?,
            )),
            1760724339 => Self::IKLegBoneChain(Box::new(IKLegBoneChain::read(decoder, offset)?)),
            1874071251 => Self::TriggerEventChangePhysicsParameters(Box::new(
                TriggerEventChangePhysicsParameters::read(decoder, offset)?,
            )),
            1886370288 => Self::TriggerEventTransparency(Box::new(TriggerEventTransparency::read(
                decoder, offset,
            )?)),
            1887511799 => Self::TriggerEventObjectLookAt(Box::new(TriggerEventObjectLookAt::read(
                decoder, offset,
            )?)),
            1921550813 => Self::TriggerEventMsg(Box::new(TriggerEventMsg::read(decoder, offset)?)),
            1976572630 => Self::TriggerEvent(Box::new(TriggerEvent::read(decoder, offset)?)),
            1984123993 => Self::ActorGizmoData(Box::new(ActorGizmoData::read(decoder, offset)?)),
            1988397685 => {
                Self::TriggerEventAddRope(Box::new(TriggerEventAddRope::read(decoder, offset)?))
            }
            2006826034 => Self::TriggerEventSetWeaponSheathed(Box::new(
                TriggerEventSetWeaponSheathed::read(decoder, offset)?,
            )),
            2145377512 => Self::TriggerEventCameraStayOnSpline(Box::new(
                TriggerEventCameraStayOnSpline::read(decoder, offset)?,
            )),
            2149480436 => Self::TriggerEventSetWeaponVisibility(Box::new(
                TriggerEventSetWeaponVisibility::read(decoder, offset)?,
            )),
            2172682041 => Self::TriggerEventCameraChangeFieldOfView(Box::new(
                TriggerEventCameraChangeFieldOfView::read(decoder, offset)?,
            )),
            2217400535 => Self::TriggerEventChangeLookDuration(Box::new(
                TriggerEventChangeLookDuration::read(decoder, offset)?,
            )),
            2239531469 => Self::TriggerEventSetOtherScenesVisibility(Box::new(
                TriggerEventSetOtherScenesVisibility::read(decoder, offset)?,
            )),
            2359108143 => Self::TriggerEventShowWarpUI(Box::new(TriggerEventShowWarpUI::read(
                decoder, offset,
            )?)),
            2372115555 => Self::TriggerEventShowCaptionUI(Box::new(
                TriggerEventShowCaptionUI::read(decoder, offset)?,
            )),
            2607134593 => Self::TriggerEventFadeMonsters(Box::new(TriggerEventFadeMonsters::read(
                decoder, offset,
            )?)),
            2627400883 => Self::SpeedTreeWind(Box::new(SpeedTreeWind::read(decoder, offset)?)),
            2649429830 => Self::TriggerEventFadePlayers(Box::new(TriggerEventFadePlayers::read(
                decoder, offset,
            )?)),
            2653177306 => Self::TriggerEventSetWeaponVisibilityDuration(Box::new(
                TriggerEventSetWeaponVisibilityDuration::read(decoder, offset)?,
            )),
            2703348374 => Self::TriggerEventSpawnObject(Box::new(TriggerEventSpawnObject::read(
                decoder, offset,
            )?)),
            2711713996 => Self::TriggerEventCameraChangeTarget(Box::new(
                TriggerEventCameraChangeTarget::read(decoder, offset)?,
            )),
            2720386370 => Self::TriggerEventEnableRagdollAnimationOverride(Box::new(
                TriggerEventEnableRagdollAnimationOverride::read(decoder, offset)?,
            )),
            2780951310 => {
                Self::SpeedTreeWindBase(Box::new(SpeedTreeWindBase::read(decoder, offset)?))
            }
            2800793622 => Self::TriggerEventDetachObject(Box::new(TriggerEventDetachObject::read(
                decoder, offset,
            )?)),
            2829109539 => Self::TriggerEventRestoreCamera(Box::new(
                TriggerEventRestoreCamera::read(decoder, offset)?,
            )),
            2863656093 => {
                Self::EffectGroupKeyData(Box::new(EffectGroupKeyData::read(decoder, offset)?))
            }
            2894373936 => Self::TriggerEventSetHelmVisibility(Box::new(
                TriggerEventSetHelmVisibility::read(decoder, offset)?,
            )),
            2926955773 => {
                Self::TriggerEventShowPoIUI(Box::new(TriggerEventShowPoIUI::read(decoder, offset)?))
            }
            3017681211 => Self::TriggerEventObjectMoveToLocator(Box::new(
                TriggerEventObjectMoveToLocator::read(decoder, offset)?,
            )),
            3019833463 => Self::TriggerEventSetRTCDebugText(Box::new(
                TriggerEventSetRTCDebugText::read(decoder, offset)?,
            )),
            3066101868 => Self::TexAnimFrameByFrameCurve(Box::new(TexAnimFrameByFrameCurve::read(
                decoder, offset,
            )?)),
            3128190323 => Self::TriggerEventCameraChangeDepthOfField(Box::new(
                TriggerEventCameraChangeDepthOfField::read(decoder, offset)?,
            )),
            3128795864 => Self::TriggerEventSetLightingLabelOverride(Box::new(
                TriggerEventSetLightingLabelOverride::read(decoder, offset)?,
            )),
            3156652577 => Self::TriggerEventObjectKeyFrame(Box::new(
                TriggerEventObjectKeyFrame::read(decoder, offset)?,
            )),
            3251137226 => Self::TriggerEventSetOcclusionShapeVisibility(Box::new(
                TriggerEventSetOcclusionShapeVisibility::read(decoder, offset)?,
            )),
            3268886310 => Self::TriggerEventScreenFade(Box::new(TriggerEventScreenFade::read(
                decoder, offset,
            )?)),
            3281363526 => {
                Self::TriggerEventFade(Box::new(TriggerEventFade::read(decoder, offset)?))
            }
            3281673073 => {
                Self::TriggerEventNull(Box::new(TriggerEventNull::read(decoder, offset)?))
            }
            3310662955 => Self::TriggerEventRoutingFunctionKeyFrames(Box::new(
                TriggerEventRoutingFunctionKeyFrames::read(decoder, offset)?,
            )),
            3326716277 => {
                Self::TriggerEventPOISkip(Box::new(TriggerEventPOISkip::read(decoder, offset)?))
            }
            3363235252 => Self::TriggerEventAttachCamera(Box::new(TriggerEventAttachCamera::read(
                decoder, offset,
            )?)),
            3365671474 => Self::TriggerEventOutroAllObjects(Box::new(
                TriggerEventOutroAllObjects::read(decoder, offset)?,
            )),
            3460728073 => Self::TriggerEventOutroEffectGroup(Box::new(
                TriggerEventOutroEffectGroup::read(decoder, offset)?,
            )),
            3464679862 => Self::TriggerEventSetIGCMusic(Box::new(TriggerEventSetIGCMusic::read(
                decoder, offset,
            )?)),
            3489737491 => Self::TriggerEventCameraKeyFrame(Box::new(
                TriggerEventCameraKeyFrame::read(decoder, offset)?,
            )),
            3493835382 => Self::TriggerEventObjectTurnToOriginalFacing(Box::new(
                TriggerEventObjectTurnToOriginalFacing::read(decoder, offset)?,
            )),
            3503113420 => {
                Self::TriggerEventPlayMovie(Box::new(TriggerEventPlayMovie::read(decoder, offset)?))
            }
            3507772913 => Self::TriggerEventChangeLook(Box::new(TriggerEventChangeLook::read(
                decoder, offset,
            )?)),
            3573101747 => {
                Self::TriggerEventFadeGroup(Box::new(TriggerEventFadeGroup::read(decoder, offset)?))
            }
            3594004500 => Self::TriggerEventChangeAttachmentHardpointDuration(Box::new(
                TriggerEventChangeAttachmentHardpointDuration::read(decoder, offset)?,
            )),
            3602917158 => Self::TriggerEventFootstepTrail(Box::new(
                TriggerEventFootstepTrail::read(decoder, offset)?,
            )),
            3611725540 => Self::TriggerEventSetTimeOfDay(Box::new(TriggerEventSetTimeOfDay::read(
                decoder, offset,
            )?)),
            3661152302 => Self::TriggerEventChangeAttachmentHardpoint(Box::new(
                TriggerEventChangeAttachmentHardpoint::read(decoder, offset)?,
            )),
            3707753598 => Self::TriggerEventObjectMoveToObject(Box::new(
                TriggerEventObjectMoveToObject::read(decoder, offset)?,
            )),
            3748928721 => Self::Subcondition(Box::new(Subcondition::read(decoder, offset)?)),
            3810234018 => Self::TriggerEventEnableRagdoll(Box::new(
                TriggerEventEnableRagdoll::read(decoder, offset)?,
            )),
            3866881285 => {
                Self::TexAnimTranslation(Box::new(TexAnimTranslation::read(decoder, offset)?))
            }
            3890074365 => Self::TriggerEventCameraFollowSpline(Box::new(
                TriggerEventCameraFollowSpline::read(decoder, offset)?,
            )),
            3905611800 => Self::TriggerEventSetWeaponSheathedDuration(Box::new(
                TriggerEventSetWeaponSheathedDuration::read(decoder, offset)?,
            )),
            3932958987 => Self::TriggerEventObjectFollowSpline(Box::new(
                TriggerEventObjectFollowSpline::read(decoder, offset)?,
            )),
            4006089948 => Self::TriggerEventRestoreTimeOfDay(Box::new(
                TriggerEventRestoreTimeOfDay::read(decoder, offset)?,
            )),
            4088142735 => Self::TriggerEventPlayConversationLine(Box::new(
                TriggerEventPlayConversationLine::read(decoder, offset)?,
            )),
            4107356838 => Self::TriggerEventOutroObject(Box::new(TriggerEventOutroObject::read(
                decoder, offset,
            )?)),
            4154622810 => Self::TriggerEventListenerFollowSpline(Box::new(
                TriggerEventListenerFollowSpline::read(decoder, offset)?,
            )),
            4166874378 => Self::TriggerEventHideWarpDestinationUI(Box::new(
                TriggerEventHideWarpDestinationUI::read(decoder, offset)?,
            )),
            4171238087 => Self::TriggerEventOutroServicesDisable(Box::new(
                TriggerEventOutroServicesDisable::read(decoder, offset)?,
            )),
            4173985261 => Self::TriggerEventRangedDuration(Box::new(
                TriggerEventRangedDuration::read(decoder, offset)?,
            )),
            4273329489 => Self::TriggerEventShowWarpDestinationUI(Box::new(
                TriggerEventShowWarpDestinationUI::read(decoder, offset)?,
            )),
            other => return Err(ReadError::UnsupportedType(other)),
        })
    }
    pub(crate) fn size(&self) -> usize {
        match self {
            Self::IKGeneralBoneChain(_) => IKGeneralBoneChain::SIZE,
            Self::TriggerEventConstantFlash(_) => TriggerEventConstantFlash::SIZE,
            Self::TriggerEventFadeOtherPlayers(_) => TriggerEventFadeOtherPlayers::SIZE,
            Self::TriggerEventStoreUIIntro(_) => TriggerEventStoreUIIntro::SIZE,
            Self::TriggerEventAttachSpawnedActor(_) => TriggerEventAttachSpawnedActor::SIZE,
            Self::TexAnimTranslationCurve(_) => TexAnimTranslationCurve::SIZE,
            Self::TriggerEventRelaxCloth(_) => TriggerEventRelaxCloth::SIZE,
            Self::TriggerEventSetAudioState(_) => TriggerEventSetAudioState::SIZE,
            Self::TriggerEventObjectTurnToFace(_) => TriggerEventObjectTurnToFace::SIZE,
            Self::TriggerEventChangeShaderMapOverrideDuration(_) => {
                TriggerEventChangeShaderMapOverrideDuration::SIZE
            }
            Self::TriggerEventShadow(_) => TriggerEventShadow::SIZE,
            Self::TriggerEventIGCLightMode(_) => TriggerEventIGCLightMode::SIZE,
            Self::IKBoneChainBase(_) => IKBoneChainBase::SIZE,
            Self::TriggerEventHidePoIUI(_) => TriggerEventHidePoIUI::SIZE,
            Self::TriggerEventShadowDuration(_) => TriggerEventShadowDuration::SIZE,
            Self::TriggerEventCameraChangeNearFarPlanes(_) => {
                TriggerEventCameraChangeNearFarPlanes::SIZE
            }
            Self::TriggerEventAddObject(_) => TriggerEventAddObject::SIZE,
            Self::TriggerEventDeleteObject(_) => TriggerEventDeleteObject::SIZE,
            Self::TriggerEventActorMovement(_) => TriggerEventActorMovement::SIZE,
            Self::SpeedTreeWind8(_) => SpeedTreeWind8::SIZE,
            Self::SpeedTreeWind9(_) => SpeedTreeWind9::SIZE,
            Self::TriggerEventFoley(_) => TriggerEventFoley::SIZE,
            Self::TexAnim(_) => TexAnim::SIZE,
            Self::TriggerEventMusic(_) => TriggerEventMusic::SIZE,
            Self::TriggerEventVoice(_) => TriggerEventVoice::SIZE,
            Self::TexAnimFrameByFrame(_) => TexAnimFrameByFrame::SIZE,
            Self::TriggerEventDisableRagdollAnimationOverride(_) => {
                TriggerEventDisableRagdollAnimationOverride::SIZE
            }
            Self::TriggerEventChangeClothParameters(_) => TriggerEventChangeClothParameters::SIZE,
            Self::TriggerEventChangeShaderMapOverride(_) => {
                TriggerEventChangeShaderMapOverride::SIZE
            }
            Self::TriggerEventHitFlashDuration(_) => TriggerEventHitFlashDuration::SIZE,
            Self::TriggerEventFrostBreath(_) => TriggerEventFrostBreath::SIZE,
            Self::TriggerEventDeleteAllObjects(_) => TriggerEventDeleteAllObjects::SIZE,
            Self::TriggerEventCutsceneUIKeyFrame(_) => TriggerEventCutsceneUIKeyFrame::SIZE,
            Self::TriggerEventAddBaseMountLoadout(_) => TriggerEventAddBaseMountLoadout::SIZE,
            Self::TriggerEventSubObjectVisibility(_) => TriggerEventSubObjectVisibility::SIZE,
            Self::TriggerEventHideWarpUI(_) => TriggerEventHideWarpUI::SIZE,
            Self::TriggerEventFlappyJawOverrideDuration(_) => {
                TriggerEventFlappyJawOverrideDuration::SIZE
            }
            Self::TriggerEventAddTrail(_) => TriggerEventAddTrail::SIZE,
            Self::TriggerEventCameraMoveToCameraMarker(_) => {
                TriggerEventCameraMoveToCameraMarker::SIZE
            }
            Self::TriggerEventDisableConstraint(_) => TriggerEventDisableConstraint::SIZE,
            Self::TriggerEventCancelLook(_) => TriggerEventCancelLook::SIZE,
            Self::TriggerEventFootstep(_) => TriggerEventFootstep::SIZE,
            Self::TriggerEventCameraTransitionToObserver(_) => {
                TriggerEventCameraTransitionToObserver::SIZE
            }
            Self::TriggerEventHideCaptionUI(_) => TriggerEventHideCaptionUI::SIZE,
            Self::TriggerEventAddEffectGroup(_) => TriggerEventAddEffectGroup::SIZE,
            Self::TriggerEventActorPlayAnimation(_) => TriggerEventActorPlayAnimation::SIZE,
            Self::TriggerEventFadeRemotePlayers(_) => TriggerEventFadeRemotePlayers::SIZE,
            Self::TriggerEventIGCHideGroup(_) => TriggerEventIGCHideGroup::SIZE,
            Self::TriggerEventHitFlash(_) => TriggerEventHitFlash::SIZE,
            Self::TriggerEventFootstepDrag(_) => TriggerEventFootstepDrag::SIZE,
            Self::TriggerEventFootstepLift(_) => TriggerEventFootstepLift::SIZE,
            Self::TriggerEventDisableCollisionMessages(_) => {
                TriggerEventDisableCollisionMessages::SIZE
            }
            Self::TriggerEventSequenceEnded(_) => TriggerEventSequenceEnded::SIZE,
            Self::IKLegBoneChain(_) => IKLegBoneChain::SIZE,
            Self::TriggerEventChangePhysicsParameters(_) => {
                TriggerEventChangePhysicsParameters::SIZE
            }
            Self::TriggerEventTransparency(_) => TriggerEventTransparency::SIZE,
            Self::TriggerEventObjectLookAt(_) => TriggerEventObjectLookAt::SIZE,
            Self::TriggerEventMsg(_) => TriggerEventMsg::SIZE,
            Self::TriggerEvent(_) => TriggerEvent::SIZE,
            Self::ActorGizmoData(_) => ActorGizmoData::SIZE,
            Self::TriggerEventAddRope(_) => TriggerEventAddRope::SIZE,
            Self::TriggerEventSetWeaponSheathed(_) => TriggerEventSetWeaponSheathed::SIZE,
            Self::TriggerEventCameraStayOnSpline(_) => TriggerEventCameraStayOnSpline::SIZE,
            Self::TriggerEventSetWeaponVisibility(_) => TriggerEventSetWeaponVisibility::SIZE,
            Self::TriggerEventCameraChangeFieldOfView(_) => {
                TriggerEventCameraChangeFieldOfView::SIZE
            }
            Self::TriggerEventChangeLookDuration(_) => TriggerEventChangeLookDuration::SIZE,
            Self::TriggerEventSetOtherScenesVisibility(_) => {
                TriggerEventSetOtherScenesVisibility::SIZE
            }
            Self::TriggerEventShowWarpUI(_) => TriggerEventShowWarpUI::SIZE,
            Self::TriggerEventShowCaptionUI(_) => TriggerEventShowCaptionUI::SIZE,
            Self::TriggerEventFadeMonsters(_) => TriggerEventFadeMonsters::SIZE,
            Self::SpeedTreeWind(_) => SpeedTreeWind::SIZE,
            Self::TriggerEventFadePlayers(_) => TriggerEventFadePlayers::SIZE,
            Self::TriggerEventSetWeaponVisibilityDuration(_) => {
                TriggerEventSetWeaponVisibilityDuration::SIZE
            }
            Self::TriggerEventSpawnObject(_) => TriggerEventSpawnObject::SIZE,
            Self::TriggerEventCameraChangeTarget(_) => TriggerEventCameraChangeTarget::SIZE,
            Self::TriggerEventEnableRagdollAnimationOverride(_) => {
                TriggerEventEnableRagdollAnimationOverride::SIZE
            }
            Self::SpeedTreeWindBase(_) => SpeedTreeWindBase::SIZE,
            Self::TriggerEventDetachObject(_) => TriggerEventDetachObject::SIZE,
            Self::TriggerEventRestoreCamera(_) => TriggerEventRestoreCamera::SIZE,
            Self::EffectGroupKeyData(_) => EffectGroupKeyData::SIZE,
            Self::TriggerEventSetHelmVisibility(_) => TriggerEventSetHelmVisibility::SIZE,
            Self::TriggerEventShowPoIUI(_) => TriggerEventShowPoIUI::SIZE,
            Self::TriggerEventObjectMoveToLocator(_) => TriggerEventObjectMoveToLocator::SIZE,
            Self::TriggerEventSetRTCDebugText(_) => TriggerEventSetRTCDebugText::SIZE,
            Self::TexAnimFrameByFrameCurve(_) => TexAnimFrameByFrameCurve::SIZE,
            Self::TriggerEventCameraChangeDepthOfField(_) => {
                TriggerEventCameraChangeDepthOfField::SIZE
            }
            Self::TriggerEventSetLightingLabelOverride(_) => {
                TriggerEventSetLightingLabelOverride::SIZE
            }
            Self::TriggerEventObjectKeyFrame(_) => TriggerEventObjectKeyFrame::SIZE,
            Self::TriggerEventSetOcclusionShapeVisibility(_) => {
                TriggerEventSetOcclusionShapeVisibility::SIZE
            }
            Self::TriggerEventScreenFade(_) => TriggerEventScreenFade::SIZE,
            Self::TriggerEventFade(_) => TriggerEventFade::SIZE,
            Self::TriggerEventNull(_) => TriggerEventNull::SIZE,
            Self::TriggerEventRoutingFunctionKeyFrames(_) => {
                TriggerEventRoutingFunctionKeyFrames::SIZE
            }
            Self::TriggerEventPOISkip(_) => TriggerEventPOISkip::SIZE,
            Self::TriggerEventAttachCamera(_) => TriggerEventAttachCamera::SIZE,
            Self::TriggerEventOutroAllObjects(_) => TriggerEventOutroAllObjects::SIZE,
            Self::TriggerEventOutroEffectGroup(_) => TriggerEventOutroEffectGroup::SIZE,
            Self::TriggerEventSetIGCMusic(_) => TriggerEventSetIGCMusic::SIZE,
            Self::TriggerEventCameraKeyFrame(_) => TriggerEventCameraKeyFrame::SIZE,
            Self::TriggerEventObjectTurnToOriginalFacing(_) => {
                TriggerEventObjectTurnToOriginalFacing::SIZE
            }
            Self::TriggerEventPlayMovie(_) => TriggerEventPlayMovie::SIZE,
            Self::TriggerEventChangeLook(_) => TriggerEventChangeLook::SIZE,
            Self::TriggerEventFadeGroup(_) => TriggerEventFadeGroup::SIZE,
            Self::TriggerEventChangeAttachmentHardpointDuration(_) => {
                TriggerEventChangeAttachmentHardpointDuration::SIZE
            }
            Self::TriggerEventFootstepTrail(_) => TriggerEventFootstepTrail::SIZE,
            Self::TriggerEventSetTimeOfDay(_) => TriggerEventSetTimeOfDay::SIZE,
            Self::TriggerEventChangeAttachmentHardpoint(_) => {
                TriggerEventChangeAttachmentHardpoint::SIZE
            }
            Self::TriggerEventObjectMoveToObject(_) => TriggerEventObjectMoveToObject::SIZE,
            Self::Subcondition(_) => Subcondition::SIZE,
            Self::TriggerEventEnableRagdoll(_) => TriggerEventEnableRagdoll::SIZE,
            Self::TexAnimTranslation(_) => TexAnimTranslation::SIZE,
            Self::TriggerEventCameraFollowSpline(_) => TriggerEventCameraFollowSpline::SIZE,
            Self::TriggerEventSetWeaponSheathedDuration(_) => {
                TriggerEventSetWeaponSheathedDuration::SIZE
            }
            Self::TriggerEventObjectFollowSpline(_) => TriggerEventObjectFollowSpline::SIZE,
            Self::TriggerEventRestoreTimeOfDay(_) => TriggerEventRestoreTimeOfDay::SIZE,
            Self::TriggerEventPlayConversationLine(_) => TriggerEventPlayConversationLine::SIZE,
            Self::TriggerEventOutroObject(_) => TriggerEventOutroObject::SIZE,
            Self::TriggerEventListenerFollowSpline(_) => TriggerEventListenerFollowSpline::SIZE,
            Self::TriggerEventHideWarpDestinationUI(_) => TriggerEventHideWarpDestinationUI::SIZE,
            Self::TriggerEventOutroServicesDisable(_) => TriggerEventOutroServicesDisable::SIZE,
            Self::TriggerEventRangedDuration(_) => TriggerEventRangedDuration::SIZE,
            Self::TriggerEventShowWarpDestinationUI(_) => TriggerEventShowWarpDestinationUI::SIZE,
        }
    }
    pub(crate) fn write(
        &self,
        encoder: &mut Encoder,
        offset: usize,
        space: Space,
    ) -> Result<(), WriteError> {
        match self {
            Self::IKGeneralBoneChain(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &64253465u32, space)?;
            }
            Self::TriggerEventConstantFlash(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &133533134u32, space)?;
            }
            Self::TriggerEventFadeOtherPlayers(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &203752552u32, space)?;
            }
            Self::TriggerEventStoreUIIntro(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &266386765u32, space)?;
            }
            Self::TriggerEventAttachSpawnedActor(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &303887094u32, space)?;
            }
            Self::TexAnimTranslationCurve(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &312499914u32, space)?;
            }
            Self::TriggerEventRelaxCloth(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &330209804u32, space)?;
            }
            Self::TriggerEventSetAudioState(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &405077397u32, space)?;
            }
            Self::TriggerEventObjectTurnToFace(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &419152424u32, space)?;
            }
            Self::TriggerEventChangeShaderMapOverrideDuration(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &469541751u32, space)?;
            }
            Self::TriggerEventShadow(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &509045980u32, space)?;
            }
            Self::TriggerEventIGCLightMode(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &514913766u32, space)?;
            }
            Self::IKBoneChainBase(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &540107926u32, space)?;
            }
            Self::TriggerEventHidePoIUI(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &547442486u32, space)?;
            }
            Self::TriggerEventShadowDuration(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &551002818u32, space)?;
            }
            Self::TriggerEventCameraChangeNearFarPlanes(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &563099623u32, space)?;
            }
            Self::TriggerEventAddObject(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &568526390u32, space)?;
            }
            Self::TriggerEventDeleteObject(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &570318784u32, space)?;
            }
            Self::TriggerEventActorMovement(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &596227098u32, space)?;
            }
            Self::SpeedTreeWind8(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &804883275u32, space)?;
            }
            Self::SpeedTreeWind9(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &804883276u32, space)?;
            }
            Self::TriggerEventFoley(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &911325909u32, space)?;
            }
            Self::TexAnim(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &911604182u32, space)?;
            }
            Self::TriggerEventMusic(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &919850711u32, space)?;
            }
            Self::TriggerEventVoice(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &930297292u32, space)?;
            }
            Self::TexAnimFrameByFrame(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1011253607u32, space)?;
            }
            Self::TriggerEventDisableRagdollAnimationOverride(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1014113135u32, space)?;
            }
            Self::TriggerEventChangeClothParameters(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1036833962u32, space)?;
            }
            Self::TriggerEventChangeShaderMapOverride(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1066764433u32, space)?;
            }
            Self::TriggerEventHitFlashDuration(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1077456687u32, space)?;
            }
            Self::TriggerEventFrostBreath(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1106358970u32, space)?;
            }
            Self::TriggerEventDeleteAllObjects(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1109533772u32, space)?;
            }
            Self::TriggerEventCutsceneUIKeyFrame(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1111377282u32, space)?;
            }
            Self::TriggerEventAddBaseMountLoadout(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1131078565u32, space)?;
            }
            Self::TriggerEventSubObjectVisibility(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1134541503u32, space)?;
            }
            Self::TriggerEventHideWarpUI(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1144581000u32, space)?;
            }
            Self::TriggerEventFlappyJawOverrideDuration(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1146575562u32, space)?;
            }
            Self::TriggerEventAddTrail(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1195077723u32, space)?;
            }
            Self::TriggerEventCameraMoveToCameraMarker(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1208547332u32, space)?;
            }
            Self::TriggerEventDisableConstraint(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1288253615u32, space)?;
            }
            Self::TriggerEventCancelLook(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1310250065u32, space)?;
            }
            Self::TriggerEventFootstep(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1328668874u32, space)?;
            }
            Self::TriggerEventCameraTransitionToObserver(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1338965781u32, space)?;
            }
            Self::TriggerEventHideCaptionUI(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1367839516u32, space)?;
            }
            Self::TriggerEventAddEffectGroup(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1384905369u32, space)?;
            }
            Self::TriggerEventActorPlayAnimation(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1487816389u32, space)?;
            }
            Self::TriggerEventFadeRemotePlayers(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1524322706u32, space)?;
            }
            Self::TriggerEventIGCHideGroup(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1538719568u32, space)?;
            }
            Self::TriggerEventHitFlash(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1648188489u32, space)?;
            }
            Self::TriggerEventFootstepDrag(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1670410600u32, space)?;
            }
            Self::TriggerEventFootstepLift(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1670688473u32, space)?;
            }
            Self::TriggerEventDisableCollisionMessages(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1687293198u32, space)?;
            }
            Self::TriggerEventSequenceEnded(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1724776431u32, space)?;
            }
            Self::IKLegBoneChain(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1760724339u32, space)?;
            }
            Self::TriggerEventChangePhysicsParameters(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1874071251u32, space)?;
            }
            Self::TriggerEventTransparency(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1886370288u32, space)?;
            }
            Self::TriggerEventObjectLookAt(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1887511799u32, space)?;
            }
            Self::TriggerEventMsg(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1921550813u32, space)?;
            }
            Self::TriggerEvent(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1976572630u32, space)?;
            }
            Self::ActorGizmoData(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1984123993u32, space)?;
            }
            Self::TriggerEventAddRope(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &1988397685u32, space)?;
            }
            Self::TriggerEventSetWeaponSheathed(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2006826034u32, space)?;
            }
            Self::TriggerEventCameraStayOnSpline(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2145377512u32, space)?;
            }
            Self::TriggerEventSetWeaponVisibility(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2149480436u32, space)?;
            }
            Self::TriggerEventCameraChangeFieldOfView(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2172682041u32, space)?;
            }
            Self::TriggerEventChangeLookDuration(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2217400535u32, space)?;
            }
            Self::TriggerEventSetOtherScenesVisibility(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2239531469u32, space)?;
            }
            Self::TriggerEventShowWarpUI(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2359108143u32, space)?;
            }
            Self::TriggerEventShowCaptionUI(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2372115555u32, space)?;
            }
            Self::TriggerEventFadeMonsters(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2607134593u32, space)?;
            }
            Self::SpeedTreeWind(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2627400883u32, space)?;
            }
            Self::TriggerEventFadePlayers(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2649429830u32, space)?;
            }
            Self::TriggerEventSetWeaponVisibilityDuration(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2653177306u32, space)?;
            }
            Self::TriggerEventSpawnObject(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2703348374u32, space)?;
            }
            Self::TriggerEventCameraChangeTarget(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2711713996u32, space)?;
            }
            Self::TriggerEventEnableRagdollAnimationOverride(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2720386370u32, space)?;
            }
            Self::SpeedTreeWindBase(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2780951310u32, space)?;
            }
            Self::TriggerEventDetachObject(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2800793622u32, space)?;
            }
            Self::TriggerEventRestoreCamera(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2829109539u32, space)?;
            }
            Self::EffectGroupKeyData(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2863656093u32, space)?;
            }
            Self::TriggerEventSetHelmVisibility(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2894373936u32, space)?;
            }
            Self::TriggerEventShowPoIUI(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &2926955773u32, space)?;
            }
            Self::TriggerEventObjectMoveToLocator(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3017681211u32, space)?;
            }
            Self::TriggerEventSetRTCDebugText(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3019833463u32, space)?;
            }
            Self::TexAnimFrameByFrameCurve(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3066101868u32, space)?;
            }
            Self::TriggerEventCameraChangeDepthOfField(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3128190323u32, space)?;
            }
            Self::TriggerEventSetLightingLabelOverride(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3128795864u32, space)?;
            }
            Self::TriggerEventObjectKeyFrame(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3156652577u32, space)?;
            }
            Self::TriggerEventSetOcclusionShapeVisibility(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3251137226u32, space)?;
            }
            Self::TriggerEventScreenFade(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3268886310u32, space)?;
            }
            Self::TriggerEventFade(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3281363526u32, space)?;
            }
            Self::TriggerEventNull(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3281673073u32, space)?;
            }
            Self::TriggerEventRoutingFunctionKeyFrames(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3310662955u32, space)?;
            }
            Self::TriggerEventPOISkip(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3326716277u32, space)?;
            }
            Self::TriggerEventAttachCamera(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3363235252u32, space)?;
            }
            Self::TriggerEventOutroAllObjects(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3365671474u32, space)?;
            }
            Self::TriggerEventOutroEffectGroup(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3460728073u32, space)?;
            }
            Self::TriggerEventSetIGCMusic(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3464679862u32, space)?;
            }
            Self::TriggerEventCameraKeyFrame(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3489737491u32, space)?;
            }
            Self::TriggerEventObjectTurnToOriginalFacing(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3493835382u32, space)?;
            }
            Self::TriggerEventPlayMovie(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3503113420u32, space)?;
            }
            Self::TriggerEventChangeLook(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3507772913u32, space)?;
            }
            Self::TriggerEventFadeGroup(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3573101747u32, space)?;
            }
            Self::TriggerEventChangeAttachmentHardpointDuration(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3594004500u32, space)?;
            }
            Self::TriggerEventFootstepTrail(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3602917158u32, space)?;
            }
            Self::TriggerEventSetTimeOfDay(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3611725540u32, space)?;
            }
            Self::TriggerEventChangeAttachmentHardpoint(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3661152302u32, space)?;
            }
            Self::TriggerEventObjectMoveToObject(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3707753598u32, space)?;
            }
            Self::Subcondition(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3748928721u32, space)?;
            }
            Self::TriggerEventEnableRagdoll(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3810234018u32, space)?;
            }
            Self::TexAnimTranslation(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3866881285u32, space)?;
            }
            Self::TriggerEventCameraFollowSpline(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3890074365u32, space)?;
            }
            Self::TriggerEventSetWeaponSheathedDuration(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3905611800u32, space)?;
            }
            Self::TriggerEventObjectFollowSpline(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &3932958987u32, space)?;
            }
            Self::TriggerEventRestoreTimeOfDay(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &4006089948u32, space)?;
            }
            Self::TriggerEventPlayConversationLine(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &4088142735u32, space)?;
            }
            Self::TriggerEventOutroObject(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &4107356838u32, space)?;
            }
            Self::TriggerEventListenerFollowSpline(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &4154622810u32, space)?;
            }
            Self::TriggerEventHideWarpDestinationUI(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &4166874378u32, space)?;
            }
            Self::TriggerEventOutroServicesDisable(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &4171238087u32, space)?;
            }
            Self::TriggerEventRangedDuration(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &4173985261u32, space)?;
            }
            Self::TriggerEventShowWarpDestinationUI(v) => {
                v.write(encoder, offset, space)?;
                encoder.value(offset, &4273329489u32, space)?;
            }
        }
        Ok(())
    }

    pub(crate) fn type_hash(&self) -> u32 {
        match self {
            Self::IKGeneralBoneChain(_) => 64253465,
            Self::TriggerEventConstantFlash(_) => 133533134,
            Self::TriggerEventFadeOtherPlayers(_) => 203752552,
            Self::TriggerEventStoreUIIntro(_) => 266386765,
            Self::TriggerEventAttachSpawnedActor(_) => 303887094,
            Self::TexAnimTranslationCurve(_) => 312499914,
            Self::TriggerEventRelaxCloth(_) => 330209804,
            Self::TriggerEventSetAudioState(_) => 405077397,
            Self::TriggerEventObjectTurnToFace(_) => 419152424,
            Self::TriggerEventChangeShaderMapOverrideDuration(_) => 469541751,
            Self::TriggerEventShadow(_) => 509045980,
            Self::TriggerEventIGCLightMode(_) => 514913766,
            Self::IKBoneChainBase(_) => 540107926,
            Self::TriggerEventHidePoIUI(_) => 547442486,
            Self::TriggerEventShadowDuration(_) => 551002818,
            Self::TriggerEventCameraChangeNearFarPlanes(_) => 563099623,
            Self::TriggerEventAddObject(_) => 568526390,
            Self::TriggerEventDeleteObject(_) => 570318784,
            Self::TriggerEventActorMovement(_) => 596227098,
            Self::SpeedTreeWind8(_) => 804883275,
            Self::SpeedTreeWind9(_) => 804883276,
            Self::TriggerEventFoley(_) => 911325909,
            Self::TexAnim(_) => 911604182,
            Self::TriggerEventMusic(_) => 919850711,
            Self::TriggerEventVoice(_) => 930297292,
            Self::TexAnimFrameByFrame(_) => 1011253607,
            Self::TriggerEventDisableRagdollAnimationOverride(_) => 1014113135,
            Self::TriggerEventChangeClothParameters(_) => 1036833962,
            Self::TriggerEventChangeShaderMapOverride(_) => 1066764433,
            Self::TriggerEventHitFlashDuration(_) => 1077456687,
            Self::TriggerEventFrostBreath(_) => 1106358970,
            Self::TriggerEventDeleteAllObjects(_) => 1109533772,
            Self::TriggerEventCutsceneUIKeyFrame(_) => 1111377282,
            Self::TriggerEventAddBaseMountLoadout(_) => 1131078565,
            Self::TriggerEventSubObjectVisibility(_) => 1134541503,
            Self::TriggerEventHideWarpUI(_) => 1144581000,
            Self::TriggerEventFlappyJawOverrideDuration(_) => 1146575562,
            Self::TriggerEventAddTrail(_) => 1195077723,
            Self::TriggerEventCameraMoveToCameraMarker(_) => 1208547332,
            Self::TriggerEventDisableConstraint(_) => 1288253615,
            Self::TriggerEventCancelLook(_) => 1310250065,
            Self::TriggerEventFootstep(_) => 1328668874,
            Self::TriggerEventCameraTransitionToObserver(_) => 1338965781,
            Self::TriggerEventHideCaptionUI(_) => 1367839516,
            Self::TriggerEventAddEffectGroup(_) => 1384905369,
            Self::TriggerEventActorPlayAnimation(_) => 1487816389,
            Self::TriggerEventFadeRemotePlayers(_) => 1524322706,
            Self::TriggerEventIGCHideGroup(_) => 1538719568,
            Self::TriggerEventHitFlash(_) => 1648188489,
            Self::TriggerEventFootstepDrag(_) => 1670410600,
            Self::TriggerEventFootstepLift(_) => 1670688473,
            Self::TriggerEventDisableCollisionMessages(_) => 1687293198,
            Self::TriggerEventSequenceEnded(_) => 1724776431,
            Self::IKLegBoneChain(_) => 1760724339,
            Self::TriggerEventChangePhysicsParameters(_) => 1874071251,
            Self::TriggerEventTransparency(_) => 1886370288,
            Self::TriggerEventObjectLookAt(_) => 1887511799,
            Self::TriggerEventMsg(_) => 1921550813,
            Self::TriggerEvent(_) => 1976572630,
            Self::ActorGizmoData(_) => 1984123993,
            Self::TriggerEventAddRope(_) => 1988397685,
            Self::TriggerEventSetWeaponSheathed(_) => 2006826034,
            Self::TriggerEventCameraStayOnSpline(_) => 2145377512,
            Self::TriggerEventSetWeaponVisibility(_) => 2149480436,
            Self::TriggerEventCameraChangeFieldOfView(_) => 2172682041,
            Self::TriggerEventChangeLookDuration(_) => 2217400535,
            Self::TriggerEventSetOtherScenesVisibility(_) => 2239531469,
            Self::TriggerEventShowWarpUI(_) => 2359108143,
            Self::TriggerEventShowCaptionUI(_) => 2372115555,
            Self::TriggerEventFadeMonsters(_) => 2607134593,
            Self::SpeedTreeWind(_) => 2627400883,
            Self::TriggerEventFadePlayers(_) => 2649429830,
            Self::TriggerEventSetWeaponVisibilityDuration(_) => 2653177306,
            Self::TriggerEventSpawnObject(_) => 2703348374,
            Self::TriggerEventCameraChangeTarget(_) => 2711713996,
            Self::TriggerEventEnableRagdollAnimationOverride(_) => 2720386370,
            Self::SpeedTreeWindBase(_) => 2780951310,
            Self::TriggerEventDetachObject(_) => 2800793622,
            Self::TriggerEventRestoreCamera(_) => 2829109539,
            Self::EffectGroupKeyData(_) => 2863656093,
            Self::TriggerEventSetHelmVisibility(_) => 2894373936,
            Self::TriggerEventShowPoIUI(_) => 2926955773,
            Self::TriggerEventObjectMoveToLocator(_) => 3017681211,
            Self::TriggerEventSetRTCDebugText(_) => 3019833463,
            Self::TexAnimFrameByFrameCurve(_) => 3066101868,
            Self::TriggerEventCameraChangeDepthOfField(_) => 3128190323,
            Self::TriggerEventSetLightingLabelOverride(_) => 3128795864,
            Self::TriggerEventObjectKeyFrame(_) => 3156652577,
            Self::TriggerEventSetOcclusionShapeVisibility(_) => 3251137226,
            Self::TriggerEventScreenFade(_) => 3268886310,
            Self::TriggerEventFade(_) => 3281363526,
            Self::TriggerEventNull(_) => 3281673073,
            Self::TriggerEventRoutingFunctionKeyFrames(_) => 3310662955,
            Self::TriggerEventPOISkip(_) => 3326716277,
            Self::TriggerEventAttachCamera(_) => 3363235252,
            Self::TriggerEventOutroAllObjects(_) => 3365671474,
            Self::TriggerEventOutroEffectGroup(_) => 3460728073,
            Self::TriggerEventSetIGCMusic(_) => 3464679862,
            Self::TriggerEventCameraKeyFrame(_) => 3489737491,
            Self::TriggerEventObjectTurnToOriginalFacing(_) => 3493835382,
            Self::TriggerEventPlayMovie(_) => 3503113420,
            Self::TriggerEventChangeLook(_) => 3507772913,
            Self::TriggerEventFadeGroup(_) => 3573101747,
            Self::TriggerEventChangeAttachmentHardpointDuration(_) => 3594004500,
            Self::TriggerEventFootstepTrail(_) => 3602917158,
            Self::TriggerEventSetTimeOfDay(_) => 3611725540,
            Self::TriggerEventChangeAttachmentHardpoint(_) => 3661152302,
            Self::TriggerEventObjectMoveToObject(_) => 3707753598,
            Self::Subcondition(_) => 3748928721,
            Self::TriggerEventEnableRagdoll(_) => 3810234018,
            Self::TexAnimTranslation(_) => 3866881285,
            Self::TriggerEventCameraFollowSpline(_) => 3890074365,
            Self::TriggerEventSetWeaponSheathedDuration(_) => 3905611800,
            Self::TriggerEventObjectFollowSpline(_) => 3932958987,
            Self::TriggerEventRestoreTimeOfDay(_) => 4006089948,
            Self::TriggerEventPlayConversationLine(_) => 4088142735,
            Self::TriggerEventOutroObject(_) => 4107356838,
            Self::TriggerEventListenerFollowSpline(_) => 4154622810,
            Self::TriggerEventHideWarpDestinationUI(_) => 4166874378,
            Self::TriggerEventOutroServicesDisable(_) => 4171238087,
            Self::TriggerEventRangedDuration(_) => 4173985261,
            Self::TriggerEventShowWarpDestinationUI(_) => 4273329489,
        }
    }
    pub(crate) fn supports_hash(base: u32, hash: u32) -> bool {
        if base == hash {
            return true;
        }
        match base {
            2780951310 => matches!(hash, 804883275 | 804883276 | 2627400883 | 2780951310),
            911604182 => matches!(
                hash,
                312499914 | 911604182 | 1011253607 | 3066101868 | 3866881285
            ),
            540107926 => matches!(hash, 64253465 | 540107926 | 1760724339),
            1976572630 => matches!(
                hash,
                133533134
                    | 203752552
                    | 266386765
                    | 303887094
                    | 330209804
                    | 405077397
                    | 419152424
                    | 469541751
                    | 509045980
                    | 514913766
                    | 547442486
                    | 551002818
                    | 563099623
                    | 568526390
                    | 570318784
                    | 596227098
                    | 911325909
                    | 919850711
                    | 930297292
                    | 1014113135
                    | 1036833962
                    | 1066764433
                    | 1077456687
                    | 1106358970
                    | 1109533772
                    | 1111377282
                    | 1131078565
                    | 1134541503
                    | 1144581000
                    | 1146575562
                    | 1195077723
                    | 1208547332
                    | 1288253615
                    | 1310250065
                    | 1328668874
                    | 1338965781
                    | 1367839516
                    | 1384905369
                    | 1487816389
                    | 1524322706
                    | 1538719568
                    | 1648188489
                    | 1670410600
                    | 1670688473
                    | 1687293198
                    | 1724776431
                    | 1874071251
                    | 1886370288
                    | 1887511799
                    | 1921550813
                    | 1976572630
                    | 1988397685
                    | 2006826034
                    | 2145377512
                    | 2149480436
                    | 2172682041
                    | 2217400535
                    | 2239531469
                    | 2359108143
                    | 2372115555
                    | 2607134593
                    | 2649429830
                    | 2653177306
                    | 2703348374
                    | 2711713996
                    | 2720386370
                    | 2800793622
                    | 2829109539
                    | 2894373936
                    | 2926955773
                    | 3017681211
                    | 3019833463
                    | 3128190323
                    | 3128795864
                    | 3156652577
                    | 3251137226
                    | 3268886310
                    | 3281363526
                    | 3281673073
                    | 3310662955
                    | 3326716277
                    | 3363235252
                    | 3365671474
                    | 3460728073
                    | 3464679862
                    | 3489737491
                    | 3493835382
                    | 3503113420
                    | 3507772913
                    | 3573101747
                    | 3594004500
                    | 3602917158
                    | 3611725540
                    | 3661152302
                    | 3707753598
                    | 3810234018
                    | 3890074365
                    | 3905611800
                    | 3932958987
                    | 4006089948
                    | 4088142735
                    | 4107356838
                    | 4154622810
                    | 4166874378
                    | 4171238087
                    | 4173985261
                    | 4273329489
            ),
            _ => false,
        }
    }
}
