// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

generate_element_enum!(
    /// Enum representing all of the possible values of the XEP-0107 moods.
    MoodEnum, "mood", MOOD, {
        /// Impressed with fear or apprehension; in fear; apprehensive.
        Afraid => "afraid",

        /// Astonished; confounded with fear, surprise or wonder.
        Amazed => "amazed",

        /// Inclined to love; having a propensity to love, or to sexual enjoyment; loving, fond, affectionate, passionate, lustful, sexual, etc.
        Amorous => "amorous",

        /// Displaying or feeling anger, i.e., a strong feeling of displeasure, hostility or antagonism towards someone or something, usually combined with an urge to harm.
        Angry => "angry",

        /// To be disturbed or irritated, especially by continued or repeated acts.
        Annoyed => "annoyed",

        /// Full of anxiety or disquietude; greatly concerned or solicitous, esp. respecting something future or unknown; being in painful suspense.
        Anxious => "anxious",

        /// To be stimulated in one's feelings, especially to be sexually stimulated.
        Aroused => "aroused",

        /// Feeling shame or guilt.
        Ashamed => "ashamed",

        /// Suffering from boredom; uninterested, without attention.
        Bored => "bored",

        /// Strong in the face of fear; courageous.
        Brave => "brave",

        /// Peaceful, quiet.
        Calm => "calm",

        /// Taking care or caution; tentative.
        Cautious => "cautious",

        /// Feeling the sensation of coldness, especially to the point of discomfort.
        Cold => "cold",

        /// Feeling very sure of or positive about something, especially about one's own capabilities.
        Confident => "confident",

        /// Chaotic, jumbled or muddled.
        Confused => "confused",

        /// Feeling introspective or thoughtful.
        Contemplative => "contemplative",

        /// Pleased at the satisfaction of a want or desire; satisfied.
        Contented => "contented",

        /// Grouchy, irritable; easily upset.
        Cranky => "cranky",

        /// Feeling out of control; feeling overly excited or enthusiastic.
        Crazy => "crazy",

        /// Feeling original, expressive, or imaginative.
        Creative => "creative",

        /// Inquisitive; tending to ask questions, investigate, or explore.
        Curious => "curious",

        /// Feeling sad and dispirited.
        Dejected => "dejected",

        /// Severely despondent and unhappy.
        Depressed => "depressed",

        /// Defeated of expectation or hope; let down.
        Disappointed => "disappointed",

        /// Filled with disgust; irritated and out of patience.
        Disgusted => "disgusted",

        /// Feeling a sudden or complete loss of courage in the face of trouble or danger.
        Dismayed => "dismayed",

        /// Having one's attention diverted; preoccupied.
        Distracted => "distracted",

        /// Having a feeling of shameful discomfort.
        Embarrassed => "embarrassed",

        /// Feeling pain by the excellence or good fortune of another.
        Envious => "envious",

        /// Having great enthusiasm.
        Excited => "excited",

        /// In the mood for flirting.
        Flirtatious => "flirtatious",

        /// Suffering from frustration; dissatisfied, agitated, or discontented because one is unable to perform an action or fulfill a desire.
        Frustrated => "frustrated",

        /// Feeling appreciation or thanks.
        Grateful => "grateful",

        /// Feeling very sad about something, especially something lost; mournful; sorrowful.
        Grieving => "grieving",

        /// Unhappy and irritable.
        Grumpy => "grumpy",

        /// Feeling responsible for wrongdoing; feeling blameworthy.
        Guilty => "guilty",

        /// Experiencing the effect of favourable fortune; having the feeling arising from the consciousness of well-being or of enjoyment; enjoying good of any kind, as peace, tranquillity, comfort; contented; joyous.
        Happy => "happy",

        /// Having a positive feeling, belief, or expectation that something wished for can or will happen.
        Hopeful => "hopeful",

        /// Feeling the sensation of heat, especially to the point of discomfort.
        Hot => "hot",

        /// Having or showing a modest or low estimate of one's own importance; feeling lowered in dignity or importance.
        Humbled => "humbled",

        /// Feeling deprived of dignity or self-respect.
        Humiliated => "humiliated",

        /// Having a physical need for food.
        Hungry => "hungry",

        /// Wounded, injured, or pained, whether physically or emotionally.
        Hurt => "hurt",

        /// Favourably affected by something or someone.
        Impressed => "impressed",

        /// Feeling amazement at something or someone; or feeling a combination of fear and reverence.
        InAwe => "in_awe",

        /// Feeling strong affection, care, liking, or attraction..
        InLove => "in_love",

        /// Showing anger or indignation, especially at something unjust or wrong.
        Indignant => "indignant",

        /// Showing great attention to something or someone; having or showing interest.
        Interested => "interested",

        /// Under the influence of alcohol; drunk.
        Intoxicated => "intoxicated",

        /// Feeling as if one cannot be defeated, overcome or denied.
        Invincible => "invincible",

        /// Fearful of being replaced in position or affection.
        Jealous => "jealous",

        /// Feeling isolated, empty, or abandoned.
        Lonely => "lonely",

        /// Unable to find one's way, either physically or emotionally.
        Lost => "lost",

        /// Feeling as if one will be favored by luck.
        Lucky => "lucky",

        /// Causing or intending to cause intentional harm; bearing ill will towards another; cruel; malicious.
        Mean => "mean",

        /// Given to sudden or frequent changes of mind or feeling; temperamental.
        Moody => "moody",

        /// Easily agitated or alarmed; apprehensive or anxious.
        Nervous => "nervous",

        /// Not having a strong mood or emotional state.
        Neutral => "neutral",

        /// Feeling emotionally hurt, displeased, or insulted.
        Offended => "offended",

        /// Feeling resentful anger caused by an extremely violent or vicious attack, or by an offensive, immoral, or indecent act.
        Outraged => "outraged",

        /// Interested in play; fun, recreational, unserious, lighthearted; joking, silly.
        Playful => "playful",

        /// Feeling a sense of one's own worth or accomplishment.
        Proud => "proud",

        /// Having an easy-going mood; not stressed; calm.
        Relaxed => "relaxed",

        /// Feeling uplifted because of the removal of stress or discomfort.
        Relieved => "relieved",

        /// Feeling regret or sadness for doing something wrong.
        Remorseful => "remorseful",

        /// Without rest; unable to be still or quiet; uneasy; continually moving.
        Restless => "restless",

        /// Feeling sorrow; sorrowful, mournful.
        Sad => "sad",

        /// Mocking and ironical.
        Sarcastic => "sarcastic",

        /// Pleased at the fulfillment of a need or desire.
        Satisfied => "satisfied",

        /// Without humor or expression of happiness; grave in manner or disposition; earnest; thoughtful; solemn.
        Serious => "serious",

        /// Surprised, startled, confused, or taken aback.
        Shocked => "shocked",

        /// Feeling easily frightened or scared; timid; reserved or coy.
        Shy => "shy",

        /// Feeling in poor health; ill.
        Sick => "sick",

        /// Feeling the need for sleep.
        Sleepy => "sleepy",

        /// Acting without planning; natural; impulsive.
        Spontaneous => "spontaneous",

        /// Suffering emotional pressure.
        Stressed => "stressed",

        /// Capable of producing great physical force; or, emotionally forceful, able, determined, unyielding.
        Strong => "strong",

        /// Experiencing a feeling caused by something unexpected.
        Surprised => "surprised",

        /// Showing appreciation or gratitude.
        Thankful => "thankful",

        /// Feeling the need to drink.
        Thirsty => "thirsty",

        /// In need of rest or sleep.
        Tired => "tired",

        /// [Feeling any emotion not defined here.]
        Undefined => "undefined",

        /// Lacking in force or ability, either physical or emotional.
        Weak => "weak",

        /// Thinking about unpleasant things that have happened or that might happen; feeling afraid and unhappy.
        Worried => "worried",
    }
);

generate_elem_id!(
    /// Free-form text description of the mood.
    Text,
    "text",
    MOOD
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(MoodEnum, 1);
        assert_size!(Text, 12);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(MoodEnum, 1);
        assert_size!(Text, 24);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<happy xmlns='http://jabber.org/protocol/mood'/>"
            .parse()
            .unwrap();
        let mood = MoodEnum::try_from(elem).unwrap();
        assert_eq!(mood, MoodEnum::Happy);
    }

    #[test]
    fn test_text() {
        let elem: Element = "<text xmlns='http://jabber.org/protocol/mood'>Yay!</text>"
            .parse()
            .unwrap();
        let elem2 = elem.clone();
        let text = Text::try_from(elem).unwrap();
        assert_eq!(text.0, String::from("Yay!"));

        let elem3 = text.into();
        assert_eq!(elem2, elem3);
    }
}
